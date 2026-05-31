use bevy::prelude::*;

use crate::components::{Apothecary, Contaminant, LevelEntity, Wall};
use crate::geometry::circle_hits_any_wall;
use crate::resources::{ApothecaryVitals, ContaminantSpawnTimer};

const APOTHECARY_RADIUS: f32 = 22.0;
const CONTAMINANT_RADIUS: f32 = 18.0;
const CONTAMINANT_SPEED: f32 = 92.0;

pub fn spawn_contaminants(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ContaminantSpawnTimer>,
) {
    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    }

    let spawn_index = timer.0.times_finished_this_tick() as f32;
    let x = if spawn_index.rem_euclid(2.0) == 0.0 {
        -405.0
    } else {
        405.0
    };
    let y = if spawn_index.rem_euclid(3.0) == 0.0 {
        -205.0
    } else {
        205.0
    };

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.78, 0.26, 0.42),
            Vec2::splat(CONTAMINANT_RADIUS * 2.0),
        ),
        Transform::from_xyz(x, y, 15.0),
        Contaminant {
            id: None,
            health: 2,
        },
        LevelEntity,
    ));
}

pub fn move_contaminants(
    time: Res<Time>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    wall_query: Query<(&Transform, &Wall)>,
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
    contaminant_query: Query<(Entity, &Transform), With<Contaminant>>,
    mut vitals: ResMut<ApothecaryVitals>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (entity, transform) in &contaminant_query {
        let distance = apothecary_position.distance(transform.translation.xy());

        if distance <= APOTHECARY_RADIUS + CONTAMINANT_RADIUS {
            vitals.0.apply_damage(8);
            commands.entity(entity).despawn();
        }
    }
}
