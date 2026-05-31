use bevy::prelude::*;

use crate::components::{Apothecary, Contaminant, LevelEntity, Wall};
use crate::geometry::circle_hits_any_wall;
use crate::resources::{ApothecaryVitals, ContaminantSpawnTimer, GameNotice, LevelRuntime};

const APOTHECARY_RADIUS: f32 = 22.0;
const CONTAMINANT_RADIUS: f32 = 18.0;
const CONTAMINANT_SPEED: f32 = 92.0;
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
    sprite.custom_size = Some(Vec2::new(54.0, 44.0));

    commands.spawn((
        sprite,
        Transform::from_xyz(position.x, position.y, 15.0),
        Contaminant {
            id: None,
            health: 2,
            hit_flash: Timer::from_seconds(0.0, TimerMode::Once),
        },
        LevelEntity,
    ));
}

pub fn move_contaminants(
    time: Res<Time>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    wall_query: Query<(&Transform, &Wall), Without<Contaminant>>,
    mut contaminant_query: Query<&mut Transform, (With<Contaminant>, Without<Apothecary>)>,
) {
    let target = apothecary_query.translation.xy();

    for mut transform in &mut contaminant_query {
        let current = transform.translation.xy();
        let direction = (target - transform.translation.xy()).normalize_or_zero();
        let delta = direction * CONTAMINANT_SPEED * time.delta_secs();
        let next = current + delta;

        if !circle_hits_any_wall(next, CONTAMINANT_RADIUS, &wall_query) {
            transform.translation = next.extend(transform.translation.z);
            continue;
        }

        let x_only = Vec2::new(next.x, current.y);
        if !circle_hits_any_wall(x_only, CONTAMINANT_RADIUS, &wall_query) {
            transform.translation.x = x_only.x;
        }

        let y_only = Vec2::new(transform.translation.x, next.y);
        if !circle_hits_any_wall(y_only, CONTAMINANT_RADIUS, &wall_query) {
            transform.translation.y = y_only.y;
        }
    }
}

pub fn resolve_contaminant_contact(
    mut commands: Commands,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    contaminant_query: Query<(Entity, &Transform), (With<Contaminant>, Without<Apothecary>)>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut notice: ResMut<GameNotice>,
) {
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
