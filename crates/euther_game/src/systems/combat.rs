use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Apothecary, Contaminant, LevelEntity, Projectile, Wall};
use crate::geometry::circle_hits_any_wall;
use crate::resources::ApothecaryVitals;

const PROJECTILE_SPEED: f32 = 720.0;
const PROJECTILE_LIFETIME: f32 = 1.1;
const CONTAMINANT_RADIUS: f32 = 18.0;
const PROJECTILE_RADIUS: f32 = 5.0;

pub fn fire_syringe_round(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    mut vitals: ResMut<ApothecaryVitals>,
) {
    if !buttons.just_pressed(MouseButton::Left) || vitals.0.ammo <= 0 {
        return;
    }

    let (camera, camera_transform) = *camera_query;
    let Some(cursor_position) = window_query.cursor_position() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let origin = apothecary_query.translation.xy();
    let direction = (world_position - origin).normalize_or_zero();

    if direction == Vec2::ZERO {
        return;
    }

    vitals.0.spend_round();

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.90, 0.98, 0.76),
            Vec2::splat(PROJECTILE_RADIUS * 2.0),
        ),
        Transform::from_xyz(
            origin.x + direction.x * 28.0,
            origin.y + direction.y * 28.0,
            20.0,
        ),
        Projectile {
            velocity: direction * PROJECTILE_SPEED,
            lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
        },
        LevelEntity,
    ));
}

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    wall_query: Query<(&Transform, &Wall)>,
    mut query: Query<(Entity, &mut Projectile, &mut Transform)>,
) {
    for (entity, mut projectile, mut transform) in &mut query {
        projectile.lifetime.tick(time.delta());
        transform.translation += (projectile.velocity * time.delta_secs()).extend(0.0);

        if projectile.lifetime.is_finished()
            || circle_hits_any_wall(transform.translation.xy(), PROJECTILE_RADIUS, &wall_query)
        {
            commands.entity(entity).despawn();
        }
    }
}

pub fn resolve_projectile_hits(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    mut contaminant_query: Query<(Entity, &Transform, &mut Contaminant)>,
    mut vitals: ResMut<ApothecaryVitals>,
) {
    for (projectile_entity, projectile_transform) in &projectile_query {
        for (contaminant_entity, contaminant_transform, mut contaminant) in &mut contaminant_query {
            let distance = projectile_transform
                .translation
                .xy()
                .distance(contaminant_transform.translation.xy());

            if distance > PROJECTILE_RADIUS + CONTAMINANT_RADIUS {
                continue;
            }

            contaminant.health -= 1;
            commands.entity(projectile_entity).despawn();

            if contaminant.health <= 0 {
                commands.entity(contaminant_entity).despawn();
                vitals.0.collect_bio_sample();
            }

            break;
        }
    }
}
