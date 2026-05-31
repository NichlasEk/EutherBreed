use bevy::prelude::*;
use game_core::{LevelDefinition, PickupKind, TerminalKind};

use crate::components::{
    Apothecary, Contaminant, Door, ExitZone, LevelEntity, NoticeText, Pickup, StatusText, Terminal,
    Wall,
};
use crate::resources::{CampaignRuntime, LevelRuntime, LocalLevelState};

pub fn setup(
    mut commands: Commands,
    campaign_runtime: Res<CampaignRuntime>,
    level_state: Res<LocalLevelState>,
    mut level_runtime: ResMut<LevelRuntime>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("Health 100 | Reagent rounds 48 | Bio-samples 0"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.78, 0.92, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
        StatusText,
    ));

    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.82, 0.46)),
        Node {
            position_type: PositionType::Absolute,
            right: px(18),
            bottom: px(18),
            ..default()
        },
        NoticeText,
    ));

    let current_level_id = campaign_runtime.progress.current_level().to_string();
    let level = load_level_from_campaign(&campaign_runtime, &current_level_id);
    spawn_level(&mut commands, &level, &level_state.0, None, None);

    level_runtime.loaded_level_id = Some(current_level_id);
}

pub fn load_level_from_campaign(
    campaign_runtime: &CampaignRuntime,
    level_id: &str,
) -> LevelDefinition {
    let campaign_level = campaign_runtime
        .definition
        .levels
        .iter()
        .find(|level| level.id == level_id)
        .unwrap_or_else(|| panic!("campaign level not found: {level_id}"));

    let level = LevelDefinition::from_ron_file(&campaign_level.path)
        .unwrap_or_else(|error| panic!("failed to load level {}: {error:?}", campaign_level.path));

    level
        .validate()
        .unwrap_or_else(|error| panic!("invalid level {}: {error:?}", campaign_level.path));

    level
}

pub fn spawn_level(
    commands: &mut Commands,
    level: &LevelDefinition,
    level_state: &game_core::LevelState,
    entry_id: Option<&str>,
    run_position: Option<Vec2>,
) {
    let apothecary_start =
        run_position.unwrap_or_else(|| apothecary_spawn_position(level, entry_id));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.08, 0.10, 0.13), Vec2::new(900.0, 520.0)),
        Transform::from_xyz(0.0, 0.0, -10.0),
        LevelEntity,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.45, 0.85, 0.72), Vec2::new(34.0, 48.0)),
        Transform::from_xyz(apothecary_start.x, apothecary_start.y, 10.0),
        Apothecary,
        LevelEntity,
    ));

    for wall in &level.walls {
        spawn_wall(commands, wall.center, wall.half_extents * 2.0);
    }

    for contaminant in &level.contaminants {
        if level_state.has_killed_contaminant(&contaminant.id) {
            continue;
        }

        spawn_contaminant(commands, Some(contaminant.id.clone()), contaminant.position);
    }

    for pickup in &level.pickups {
        if level_state.has_collected_pickup(&pickup.id) {
            continue;
        }

        spawn_pickup(
            commands,
            pickup.id.clone(),
            pickup.position,
            pickup.kind.clone(),
        );
    }

    for door in &level.doors {
        let locked = door.starts_locked
            && !level_state.has_unlocked_door(&door.id)
            && !level_state.has_clearance(&door.clearance_id);
        spawn_door(
            commands,
            door.id.clone(),
            door.position,
            door.half_extents * 2.0,
            door.clearance_id.clone(),
            locked,
        );
    }

    for terminal in &level.terminals {
        spawn_terminal(
            commands,
            terminal.id.clone(),
            terminal.position,
            terminal.kind.clone(),
            terminal.objective_id.clone(),
        );
    }

    for exit in &level.exits {
        commands.spawn((
            Sprite::from_color(
                Color::srgba(0.30, 0.72, 0.95, 0.55),
                exit.half_extents * 2.0,
            ),
            Transform::from_xyz(exit.position.x, exit.position.y, 2.0),
            ExitZone {
                target: exit.target.clone(),
                entry_id: exit.entry_id.clone(),
                required_objectives: exit.required_objectives.clone(),
            },
            LevelEntity,
        ));
    }
}

pub fn apothecary_spawn_position(level: &LevelDefinition, entry_id: Option<&str>) -> Vec2 {
    entry_id
        .and_then(|entry_id| {
            level
                .entry_points
                .iter()
                .find(|entry_point| entry_point.id == entry_id)
        })
        .map(|entry_point| entry_point.position)
        .unwrap_or(level.apothecary_start)
}

fn spawn_wall(commands: &mut Commands, center: Vec2, size: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.23, 0.28, 0.32), size),
        Transform::from_xyz(center.x, center.y, -5.0),
        Wall {
            half_extents: size * 0.5,
        },
        LevelEntity,
    ));
}

fn spawn_contaminant(commands: &mut Commands, id: Option<String>, position: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.78, 0.26, 0.42), Vec2::splat(36.0)),
        Transform::from_xyz(position.x, position.y, 15.0),
        Contaminant { id, health: 2 },
        LevelEntity,
    ));
}

fn spawn_pickup(commands: &mut Commands, id: String, position: Vec2, kind: PickupKind) {
    let color = match kind {
        PickupKind::ReagentRounds(_) => Color::srgb(0.90, 0.86, 0.42),
        PickupKind::MedGel(_) => Color::srgb(0.28, 0.88, 0.64),
        PickupKind::BioSample => Color::srgb(0.72, 0.36, 0.90),
        PickupKind::SecurityKeycard(_) => Color::srgb(0.36, 0.62, 0.96),
    };

    commands.spawn((
        Sprite::from_color(color, Vec2::splat(18.0)),
        Transform::from_xyz(position.x, position.y, 6.0),
        Pickup { id, kind },
        LevelEntity,
    ));
}

fn spawn_door(
    commands: &mut Commands,
    id: String,
    center: Vec2,
    size: Vec2,
    clearance_id: String,
    locked: bool,
) {
    let color = if locked {
        Color::srgb(0.20, 0.58, 0.62)
    } else {
        Color::srgba(0.20, 0.58, 0.62, 0.25)
    };

    let mut entity = commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(center.x, center.y, -3.0),
        Door {
            id,
            clearance_id,
            locked,
        },
        LevelEntity,
    ));

    if locked {
        entity.insert(Wall {
            half_extents: size * 0.5,
        });
    }
}

fn spawn_terminal(
    commands: &mut Commands,
    id: String,
    position: Vec2,
    kind: TerminalKind,
    objective_id: Option<String>,
) {
    let color = match kind {
        TerminalKind::LabAnalyzer => Color::srgb(0.35, 0.82, 0.72),
        TerminalKind::ShipLog => Color::srgb(0.50, 0.64, 0.95),
        TerminalKind::SupplyConsole => Color::srgb(0.84, 0.72, 0.35),
    };

    commands.spawn((
        Sprite::from_color(color, Vec2::new(30.0, 22.0)),
        Transform::from_xyz(position.x, position.y, 4.0),
        Terminal {
            id,
            kind,
            objective_id,
        },
        LevelEntity,
    ));
}
