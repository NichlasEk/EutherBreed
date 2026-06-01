use bevy::prelude::*;
use game_core::{DoorKind, PickupKind};

use crate::components::{Apothecary, Contaminant, MapOverlay};
use crate::resources::{CurrentLevelMap, GameNotice, LocalLevelState};

const MAP_SIZE: Vec2 = Vec2::new(560.0, 330.0);
const MAP_Z: f32 = 850.0;

pub fn render_map_overlay_on_shift(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    current_map: Res<CurrentLevelMap>,
    level_state: Res<LocalLevelState>,
    mut notice: ResMut<GameNotice>,
    camera_query: Single<&Transform, (With<Camera2d>, Without<MapOverlay>)>,
    apothecary_query: Query<&Transform, (With<Apothecary>, Without<Camera2d>, Without<MapOverlay>)>,
    contaminant_query: Query<
        &Transform,
        (With<Contaminant>, Without<Camera2d>, Without<MapOverlay>),
    >,
    overlay_query: Query<Entity, With<MapOverlay>>,
) {
    for entity in &overlay_query {
        commands.entity(entity).despawn();
    }

    if !(input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) {
        return;
    }

    let Some(level) = &current_map.level else {
        return;
    };

    if !level_state.0.area_scan_acquired {
        notice.show("Area scan required", 1.2);
        return;
    }

    let center = camera_query.translation.xy();
    let bounds_size = level.bounds.half_extents * 2.0;
    let scale = (MAP_SIZE.x / bounds_size.x).min(MAP_SIZE.y / bounds_size.y);

    spawn_map_rect(
        &mut commands,
        center,
        MAP_SIZE + Vec2::splat(32.0),
        MAP_Z,
        Color::srgba(0.0, 0.0, 0.0, 0.82),
    );
    spawn_map_rect(
        &mut commands,
        center,
        MAP_SIZE,
        MAP_Z + 0.1,
        Color::srgba(0.02, 0.05, 0.06, 0.92),
    );

    for wall in &level.walls {
        spawn_map_rect(
            &mut commands,
            map_position(center, level.bounds.center, wall.center, scale),
            wall.half_extents * 2.0 * scale,
            MAP_Z + 0.2,
            Color::srgba(0.34, 0.48, 0.52, 0.92),
        );
    }

    for door in &level.doors {
        let locked = door.starts_locked
            && !level_state.0.has_unlocked_door(&door.id)
            && !door_requirements_met(&door.clearance_id, &door.required_objectives, &level_state);
        spawn_map_rect(
            &mut commands,
            map_position(center, level.bounds.center, door.position, scale),
            (door.half_extents * 2.0 * scale).max(Vec2::splat(6.0)),
            MAP_Z + 0.3,
            door_map_color(door.kind, locked),
        );
    }

    for exit in &level.exits {
        spawn_map_rect(
            &mut commands,
            map_position(center, level.bounds.center, exit.position, scale),
            (exit.half_extents * 2.0 * scale).max(Vec2::splat(8.0)),
            MAP_Z + 0.35,
            Color::srgba(0.15, 1.0, 0.8, 0.95),
        );
    }

    for pickup in &level.pickups {
        if level_state.0.has_collected_pickup(&pickup.id) {
            continue;
        }
        let color = match pickup.kind {
            PickupKind::ReagentRounds(_) => Color::srgba(0.96, 0.88, 0.25, 0.95),
            PickupKind::MedGel(_) => Color::srgba(0.4, 1.0, 0.7, 0.95),
            PickupKind::BioSample => Color::srgba(0.2, 0.9, 1.0, 0.95),
            PickupKind::SecurityKeycard(_) => Color::srgba(0.95, 0.7, 0.2, 0.95),
            PickupKind::AreaScan => Color::srgba(0.25, 0.95, 1.0, 0.95),
        };
        spawn_map_rect(
            &mut commands,
            map_position(center, level.bounds.center, pickup.position, scale),
            Vec2::splat(7.0),
            MAP_Z + 0.4,
            color,
        );
    }

    for terminal in &level.terminals {
        spawn_map_rect(
            &mut commands,
            map_position(center, level.bounds.center, terminal.position, scale),
            Vec2::new(10.0, 8.0),
            MAP_Z + 0.45,
            if level_state.0.activated_terminals.contains(&terminal.id) {
                Color::srgba(0.42, 0.9, 0.78, 0.75)
            } else {
                Color::srgba(0.12, 0.8, 1.0, 0.95)
            },
        );
    }

    for transform in &contaminant_query {
        spawn_map_rect(
            &mut commands,
            map_position(
                center,
                level.bounds.center,
                transform.translation.xy(),
                scale,
            ),
            Vec2::splat(7.0),
            MAP_Z + 0.5,
            Color::srgba(1.0, 0.18, 0.38, 0.95),
        );
    }

    if let Ok(transform) = apothecary_query.single() {
        spawn_map_rect(
            &mut commands,
            map_position(
                center,
                level.bounds.center,
                transform.translation.xy(),
                scale,
            ),
            Vec2::splat(9.0),
            MAP_Z + 0.6,
            Color::srgba(0.9, 1.0, 0.95, 1.0),
        );
    }
}

fn map_position(map_center: Vec2, level_center: Vec2, world_position: Vec2, scale: f32) -> Vec2 {
    map_center + (world_position - level_center) * scale
}

fn spawn_map_rect(commands: &mut Commands, center: Vec2, size: Vec2, z: f32, color: Color) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(center.x, center.y, z),
        MapOverlay,
    ));
}

fn door_map_color(kind: DoorKind, locked: bool) -> Color {
    match (kind, locked) {
        (DoorKind::Bulkhead, true) => Color::srgba(1.0, 0.78, 0.25, 0.96),
        (DoorKind::Bulkhead, false) => Color::srgba(0.25, 0.95, 0.85, 0.88),
        (DoorKind::EnergyBarrier, true) => Color::srgba(0.85, 0.28, 1.0, 0.96),
        (DoorKind::EnergyBarrier, false) => Color::srgba(0.15, 0.70, 1.0, 0.70),
    }
}

fn door_requirements_met(
    clearance_id: &str,
    required_objectives: &[String],
    level_state: &LocalLevelState,
) -> bool {
    let clearance_met = clearance_id == "open" || level_state.0.has_clearance(clearance_id);
    let objectives_met = required_objectives
        .iter()
        .all(|objective_id| level_state.0.objectives.is_complete(objective_id));

    clearance_met && objectives_met
}
