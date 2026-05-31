use bevy::prelude::*;
use game_core::{LevelDefinition, PickupKind, TerminalKind};
use std::time::Duration;

use crate::components::{
    Apothecary, Contaminant, Door, ExitZone, LevelEntity, NoticeText, Pickup, SectionText,
    StatusText, Terminal, Wall,
};
use crate::resources::{CampaignRuntime, ContaminantSpawnTimer, LevelRuntime, LocalLevelState};

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    campaign_runtime: Res<CampaignRuntime>,
    level_state: Res<LocalLevelState>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
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

    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.58, 0.72, 0.84)),
        Node {
            position_type: PositionType::Absolute,
            top: px(44),
            left: px(16),
            ..default()
        },
        SectionText,
    ));

    let current_level_id = campaign_runtime.progress.current_level().to_string();
    let level = load_level_from_campaign(&campaign_runtime, &current_level_id);
    spawn_level(
        &mut commands,
        &asset_server,
        &level,
        &level_state.0,
        None,
        None,
    );
    update_level_runtime(&mut level_runtime, &level, &mut contaminant_timer);

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
    asset_server: &AssetServer,
    level: &LevelDefinition,
    level_state: &game_core::LevelState,
    entry_id: Option<&str>,
    run_position: Option<Vec2>,
) {
    let apothecary_start =
        run_position.unwrap_or_else(|| apothecary_spawn_position(level, entry_id));

    commands.spawn((
        image_sprite(
            asset_server,
            "sprites/biomech/tile_floor_biomech.png",
            Vec2::new(900.0, 520.0),
            level_floor_tint(&level.name),
        ),
        Transform::from_xyz(0.0, 0.0, -10.0),
        LevelEntity,
    ));

    commands.spawn((
        apothecary_sprite(asset_server),
        Transform::from_xyz(apothecary_start.x, apothecary_start.y, 10.0),
        Apothecary,
        LevelEntity,
    ));

    for wall in &level.walls {
        spawn_wall(
            commands,
            asset_server,
            wall.center,
            wall.half_extents * 2.0,
            &level.name,
        );
    }

    for contaminant in &level.contaminants {
        if level_state.has_killed_contaminant(&contaminant.id) {
            continue;
        }

        spawn_contaminant(
            commands,
            asset_server,
            Some(contaminant.id.clone()),
            contaminant.position,
        );
    }

    for pickup in &level.pickups {
        if level_state.has_collected_pickup(&pickup.id) {
            continue;
        }

        spawn_pickup(
            commands,
            asset_server,
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
            asset_server,
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
            asset_server,
            terminal.id.clone(),
            terminal.position,
            terminal.kind.clone(),
            terminal.objective_id.clone(),
        );
    }

    for exit in &level.exits {
        commands.spawn((
            image_sprite(
                asset_server,
                "sprites/biomech/exit_marker.png",
                exit.half_extents * 2.0,
                Color::srgba(0.72, 1.0, 0.95, 0.90),
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

fn apothecary_sprite(asset_server: &AssetServer) -> Sprite {
    let mut sprite = Sprite::from_image(asset_server.load("sprites/apothecary_topdown.png"));
    sprite.rect = Some(Rect::new(244.0, 348.0, 1113.0, 802.0));
    sprite.custom_size = Some(Vec2::new(96.0, 50.0));
    sprite
}

fn image_sprite(
    asset_server: &AssetServer,
    path: &'static str,
    size: Vec2,
    color: Color,
) -> Sprite {
    let mut sprite = Sprite::from_image(asset_server.load(path));
    sprite.custom_size = Some(size);
    sprite.color = color;
    sprite
}

pub fn update_level_runtime(
    level_runtime: &mut LevelRuntime,
    level: &LevelDefinition,
    contaminant_timer: &mut ContaminantSpawnTimer,
) {
    let interval = level.spawn_interval_seconds.unwrap_or(0.0);
    level_runtime.dynamic_spawn_points = level.spawn_points.clone();
    level_runtime.dynamic_spawn_cursor = 0;
    level_runtime.dynamic_spawn_interval_seconds = interval;
    level_runtime.available_exits = level.exits.iter().map(|exit| exit.target.clone()).collect();

    if interval > 0.0 {
        contaminant_timer
            .0
            .set_duration(Duration::from_secs_f32(interval));
    }
    contaminant_timer.0.reset();
}

fn spawn_wall(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    size: Vec2,
    level_name: &str,
) {
    commands.spawn((
        image_sprite(
            asset_server,
            "sprites/biomech/tile_wall_biomech.png",
            size,
            level_wall_tint(level_name),
        ),
        Transform::from_xyz(center.x, center.y, -5.0),
        Wall {
            half_extents: size * 0.5,
        },
        LevelEntity,
    ));
}

fn level_floor_tint(level_name: &str) -> Color {
    match level_name {
        "lab_access_corridor" => Color::srgba(0.22, 0.32, 0.36, 0.82),
        "triage_vault" => Color::srgba(0.30, 0.24, 0.34, 0.82),
        _ => Color::srgba(0.25, 0.30, 0.34, 0.82),
    }
}

fn level_wall_tint(level_name: &str) -> Color {
    match level_name {
        "lab_access_corridor" => Color::srgba(0.42, 0.62, 0.66, 0.92),
        "triage_vault" => Color::srgba(0.58, 0.46, 0.66, 0.92),
        _ => Color::srgba(0.50, 0.56, 0.62, 0.92),
    }
}

fn spawn_contaminant(
    commands: &mut Commands,
    asset_server: &AssetServer,
    id: Option<String>,
    position: Vec2,
) {
    commands.spawn((
        image_sprite(
            asset_server,
            "sprites/biomech/contaminant.png",
            Vec2::new(54.0, 44.0),
            Color::WHITE,
        ),
        Transform::from_xyz(position.x, position.y, 15.0),
        Contaminant {
            id,
            health: 2,
            hit_flash: Timer::from_seconds(0.0, TimerMode::Once),
        },
        LevelEntity,
    ));
}

fn spawn_pickup(
    commands: &mut Commands,
    asset_server: &AssetServer,
    id: String,
    position: Vec2,
    kind: PickupKind,
) {
    let (path, size) = match kind {
        PickupKind::ReagentRounds(_) => (
            "sprites/biomech/pickup_reagent_rounds.png",
            Vec2::new(22.0, 32.0),
        ),
        PickupKind::MedGel(_) => ("sprites/biomech/pickup_med_gel.png", Vec2::new(22.0, 34.0)),
        PickupKind::BioSample => (
            "sprites/biomech/pickup_bio_sample.png",
            Vec2::new(20.0, 40.0),
        ),
        PickupKind::SecurityKeycard(_) => (
            "sprites/biomech/pickup_security_keycard.png",
            Vec2::new(34.0, 26.0),
        ),
    };

    commands.spawn((
        image_sprite(asset_server, path, size, Color::WHITE),
        Transform::from_xyz(position.x, position.y, 6.0),
        Pickup { id, kind },
        LevelEntity,
    ));
}

fn spawn_door(
    commands: &mut Commands,
    asset_server: &AssetServer,
    id: String,
    center: Vec2,
    size: Vec2,
    clearance_id: String,
    locked: bool,
) {
    let color = if locked {
        Color::WHITE
    } else {
        Color::srgba(0.55, 0.85, 0.80, 0.36)
    };

    let mut entity = commands.spawn((
        image_sprite(
            asset_server,
            "sprites/biomech/door_quarantine.png",
            size,
            color,
        ),
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
    asset_server: &AssetServer,
    id: String,
    position: Vec2,
    kind: TerminalKind,
    objective_id: Option<String>,
) {
    let (path, color) = match kind {
        TerminalKind::LabAnalyzer => ("sprites/biomech/terminal_lab_analyzer.png", Color::WHITE),
        TerminalKind::ShipLog => (
            "sprites/biomech/terminal_lab_analyzer.png",
            Color::srgba(0.78, 0.88, 1.0, 1.0),
        ),
        TerminalKind::SupplyConsole => {
            ("sprites/biomech/terminal_supply_console.png", Color::WHITE)
        }
    };

    commands.spawn((
        image_sprite(asset_server, path, Vec2::new(42.0, 40.0), color),
        Transform::from_xyz(position.x, position.y, 4.0),
        Terminal {
            id,
            kind,
            objective_id,
        },
        LevelEntity,
    ));
}
