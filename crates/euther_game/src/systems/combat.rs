use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Apothecary, Contaminant, EffectLifetime, LevelEntity, Projectile, Wall};
use crate::geometry::segment_box_hit;
use crate::resources::{ApothecaryVitals, GameNotice, LocalLevelState};

const PROJECTILE_SPEED: f32 = 720.0;
const PROJECTILE_LIFETIME: f32 = 1.1;
const CONTAMINANT_RADIUS: f32 = 18.0;
const PROJECTILE_RADIUS: f32 = 5.0;

pub fn fire_syringe_round(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    mut vitals: ResMut<ApothecaryVitals>,
) {
    if vitals.0.health == 0 {
        return;
    }

    let fire_pressed = buttons.just_pressed(MouseButton::Left)
        || keys.just_pressed(KeyCode::KeyZ)
        || keys.just_pressed(KeyCode::KeyX)
        || keys.just_pressed(KeyCode::KeyC);

    if !fire_pressed || vitals.0.ammo <= 0 {
        return;
    }

    let (camera, camera_transform) = *camera_query;
    let origin = apothecary_query.translation.xy();
    let direction = window_query
        .cursor_position()
        .and_then(|cursor_position| {
            camera
                .viewport_to_world_2d(camera_transform, cursor_position)
                .ok()
        })
        .map(|world_position| (world_position - origin).normalize_or_zero())
        .unwrap_or_else(|| {
            (apothecary_query.rotation * Vec3::X)
                .xy()
                .normalize_or_zero()
        });

    if direction == Vec2::ZERO {
        return;
    }

    vitals.0.spend_round();

    let mut sprite =
        Sprite::from_image(asset_server.load("sprites/biomech/projectile_reagent.png"));
    sprite.custom_size = Some(Vec2::new(30.0, 9.0));

    commands.spawn((
        sprite,
        Transform::from_xyz(
            origin.x + direction.x * 28.0,
            origin.y + direction.y * 28.0,
            20.0,
        )
        .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
        Projectile {
            velocity: direction * PROJECTILE_SPEED,
            lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
        },
        LevelEntity,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgba(0.85, 1.0, 0.82, 0.82), Vec2::new(20.0, 8.0)),
        Transform::from_xyz(
            origin.x + direction.x * 22.0,
            origin.y + direction.y * 22.0,
            21.0,
        )
        .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
        EffectLifetime(Timer::from_seconds(0.07, TimerMode::Once)),
        LevelEntity,
    ));
}

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<AssetServer>,
    audio: Res<crate::audio_settings::AudioSettings>,
    mut state: ResMut<LocalLevelState>,
    wall_query: Query<(Entity, &Transform, &Wall), Without<Projectile>>,
    mut breaches: Query<(&mut crate::components::Door, &mut crate::breach::Breachable)>,
    mut query: Query<(Entity, &mut Projectile, &mut Transform)>,
) {
    for (entity, mut projectile, mut transform) in &mut query {
        projectile.lifetime.tick(time.delta());
        let from = transform.translation.xy();
        let to = from + projectile.velocity * time.delta_secs();
        // Sweep against the nearest wall so fast rounds cannot skip a thin door
        // or damage a weak door through the solid wall in front of it.
        let hit = wall_query
            .iter()
            .filter_map(|(id, t, w)| {
                segment_box_hit(
                    from,
                    to,
                    t.translation.xy(),
                    w.half_extents + Vec2::splat(PROJECTILE_RADIUS),
                )
                .map(|fraction| (id, fraction))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((wall, fraction)) = hit {
            let position = from.lerp(to, fraction);
            if let Ok((mut door, mut breach)) = breaches.get_mut(wall) {
                crate::breach::hit(
                    &mut commands,
                    &assets,
                    &audio,
                    &mut state,
                    wall,
                    &mut door,
                    &mut breach,
                    position,
                );
            }
            commands.entity(entity).despawn();
        } else {
            transform.translation.x = to.x;
            transform.translation.y = to.y;
            if projectile.lifetime.is_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn resolve_projectile_hits(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    mut contaminant_query: Query<(Entity, &Transform, &mut Contaminant, &mut Sprite)>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut level_state: ResMut<LocalLevelState>,
    mut notice: ResMut<GameNotice>,
) {
    for (projectile_entity, projectile_transform) in &projectile_query {
        for (contaminant_entity, contaminant_transform, mut contaminant, mut sprite) in
            &mut contaminant_query
        {
            let distance = projectile_transform
                .translation
                .xy()
                .distance(contaminant_transform.translation.xy());

            if distance > PROJECTILE_RADIUS + CONTAMINANT_RADIUS {
                continue;
            }

            contaminant.health -= 1;
            contaminant.hit_flash = Timer::from_seconds(0.12, TimerMode::Once);
            sprite.color = Color::srgb(1.0, 0.55, 0.52);
            commands.entity(projectile_entity).despawn();
            commands.spawn((
                Sprite::from_color(Color::srgba(0.98, 0.24, 0.38, 0.78), Vec2::splat(18.0)),
                Transform::from_xyz(
                    projectile_transform.translation.x,
                    projectile_transform.translation.y,
                    24.0,
                ),
                EffectLifetime(Timer::from_seconds(0.11, TimerMode::Once)),
                LevelEntity,
            ));

            if contaminant.health <= 0 {
                if let Some(contaminant_id) = &contaminant.id {
                    level_state.0.kill_contaminant(contaminant_id.clone());
                }

                commands.entity(contaminant_entity).despawn();
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.12, 0.95, 0.78, 0.62), Vec2::splat(30.0)),
                    Transform::from_xyz(
                        contaminant_transform.translation.x,
                        contaminant_transform.translation.y,
                        23.0,
                    ),
                    EffectLifetime(Timer::from_seconds(0.18, TimerMode::Once)),
                    LevelEntity,
                ));
                vitals.0.collect_bio_sample();
                notice.show("Contaminant neutralized", 1.4);
            }

            break;
        }
    }
}

pub fn update_effect_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_query: Query<(Entity, &mut EffectLifetime, &mut Sprite)>,
) {
    for (entity, mut lifetime, mut sprite) in &mut effect_query {
        lifetime.0.tick(time.delta());
        let remaining = lifetime.0.fraction_remaining();
        sprite.color = sprite.color.with_alpha(remaining);

        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_contaminant_hit_flash(
    time: Res<Time>,
    mut contaminant_query: Query<(&mut Contaminant, &mut Sprite)>,
) {
    for (mut contaminant, mut sprite) in &mut contaminant_query {
        if contaminant.hit_flash.is_finished() {
            continue;
        }

        contaminant.hit_flash.tick(time.delta());

        if contaminant.hit_flash.is_finished() {
            sprite.color = Color::WHITE;
        }
    }
}
