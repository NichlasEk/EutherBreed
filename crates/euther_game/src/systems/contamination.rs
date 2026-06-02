use bevy::prelude::*;

use crate::components::{Apothecary, Contaminant, ContaminantAnimation, LevelEntity, Wall};
use crate::geometry::circle_hits_any_wall;
use crate::resources::{ApothecaryVitals, ContaminantSpawnTimer, GameNotice, LevelRuntime};
use crate::setup::contaminant_animation;

const APOTHECARY_RADIUS: f32 = 22.0;
const CONTAMINANT_RADIUS: f32 = 18.0;
const CONTAMINANT_SPEED: f32 = 92.0;
const CONTAMINANT_PATROL_SPEED: f32 = 48.0;
const CONTAMINANT_AGGRO_RADIUS: f32 = 270.0;
const CONTAMINANT_RETURN_RADIUS: f32 = 88.0;
const CONTAMINANT_WALK_PHASE_PER_UNIT: f32 = 0.22;
const CONTAMINANT_WALK_WOBBLE: f32 = 0.13;
const MAX_DYNAMIC_CONTAMINANTS: usize = 4;

pub fn spawn_contaminants(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<ContaminantSpawnTimer>,
    mut level_runtime: ResMut<LevelRuntime>,
    contaminant_query: Query<&Contaminant>,
) {
    if level_runtime.dynamic_spawn_points.is_empty()
        || level_runtime.dynamic_spawn_interval_seconds <= 0.0
    {
        return;
    }

    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    }

    let dynamic_count = contaminant_query
        .iter()
        .filter(|contaminant| contaminant.id.is_none())
        .count();

    if dynamic_count >= MAX_DYNAMIC_CONTAMINANTS {
        return;
    }

    let position = level_runtime.dynamic_spawn_points
        [level_runtime.dynamic_spawn_cursor % level_runtime.dynamic_spawn_points.len()];
    level_runtime.dynamic_spawn_cursor += 1;

    let mut sprite = Sprite::from_image(asset_server.load("sprites/biomech/contaminant.png"));
    sprite.custom_size = Some(Vec2::new(64.0, 50.0));

    commands.spawn((
        sprite,
        Transform::from_xyz(position.x, position.y, 15.0),
        Contaminant {
            id: None,
            health: 2,
            hit_flash: Timer::from_seconds(0.0, TimerMode::Once),
            home_position: position,
            patrol_phase: position.x.mul_add(0.017, position.y * 0.011),
        },
        contaminant_animation(&asset_server),
        LevelEntity,
    ));
}

pub fn move_contaminants(
    time: Res<Time>,
    vitals: Res<ApothecaryVitals>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    wall_query: Query<(&Transform, &Wall), Without<Contaminant>>,
    mut contaminant_query: Query<
        (
            &mut Transform,
            &mut Sprite,
            &mut Contaminant,
            &mut ContaminantAnimation,
        ),
        (With<Contaminant>, Without<Apothecary>),
    >,
) {
    if vitals.0.health == 0 {
        return;
    }

    let target = apothecary_query.translation.xy();

    for (mut transform, mut sprite, mut contaminant, mut animation) in &mut contaminant_query {
        let current = transform.translation.xy();
        contaminant.patrol_phase += time.delta_secs();
        let target_distance = current.distance(target);
        let (direction, speed) = if target_distance <= CONTAMINANT_AGGRO_RADIUS {
            ((target - current).normalize_or_zero(), CONTAMINANT_SPEED)
        } else {
            let patrol_target = contaminant.home_position
                + Vec2::new(
                    (contaminant.patrol_phase * 0.9).cos() * CONTAMINANT_RETURN_RADIUS,
                    (contaminant.patrol_phase * 1.3).sin() * (CONTAMINANT_RETURN_RADIUS * 0.62),
                );
            (
                (patrol_target - current).normalize_or_zero(),
                CONTAMINANT_PATROL_SPEED,
            )
        };
        let delta = direction * speed * time.delta_secs();
        let next = current + delta;

        if !circle_hits_any_wall(next, CONTAMINANT_RADIUS, &wall_query) {
            transform.translation = next.extend(transform.translation.z);
            apply_contaminant_walk(
                &mut transform,
                &mut sprite,
                &mut animation,
                direction,
                delta.length(),
            );
            continue;
        }

        let mut moved = Vec2::ZERO;

        let x_only = Vec2::new(next.x, current.y);
        if !circle_hits_any_wall(x_only, CONTAMINANT_RADIUS, &wall_query) {
            transform.translation.x = x_only.x;
            moved.x = x_only.x - current.x;
        }

        let y_only = Vec2::new(transform.translation.x, next.y);
        if !circle_hits_any_wall(y_only, CONTAMINANT_RADIUS, &wall_query) {
            transform.translation.y = y_only.y;
            moved.y = y_only.y - current.y;
        }

        if moved.length_squared() > 0.001 {
            apply_contaminant_walk(
                &mut transform,
                &mut sprite,
                &mut animation,
                moved.normalize(),
                moved.length(),
            );
        } else {
            transform.scale = Vec3::ONE;
        }
    }
}

fn apply_contaminant_walk(
    transform: &mut Transform,
    sprite: &mut Sprite,
    animation: &mut ContaminantAnimation,
    direction: Vec2,
    distance: f32,
) {
    animation.phase += distance * CONTAMINANT_WALK_PHASE_PER_UNIT;

    let stride = animation.phase.sin();
    sprite.image = if stride >= 0.0 {
        animation.stride_image.clone()
    } else {
        animation.base_image.clone()
    };

    let facing = direction.y.atan2(direction.x);
    let wobble = stride * CONTAMINANT_WALK_WOBBLE;
    transform.rotation = Quat::from_rotation_z(facing + wobble);
    transform.scale = Vec3::new(1.0, 1.0 + stride.abs() * 0.04, 1.0);
}

pub fn resolve_contaminant_contact(
    mut commands: Commands,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    contaminant_query: Query<(Entity, &Transform), (With<Contaminant>, Without<Apothecary>)>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut notice: ResMut<GameNotice>,
) {
    if vitals.0.health == 0 {
        return;
    }

    let apothecary_position = apothecary_query.translation.xy();

    for (entity, transform) in &contaminant_query {
        let distance = apothecary_position.distance(transform.translation.xy());

        if distance <= APOTHECARY_RADIUS + CONTAMINANT_RADIUS {
            vitals.0.apply_damage(8);
            notice.show("Suit breach: health -8", 1.5);
            commands.entity(entity).despawn();
        }
    }
}
