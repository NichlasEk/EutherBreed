use bevy::prelude::*;
use game_core::{LevelDefinition, PickupKind, TerminalKind};

use crate::components::{
    Apothecary, Contaminant, Door, ExitZone, Pickup, StatusText, Terminal, Wall,
};

pub fn setup(mut commands: Commands) {
    let level = LevelDefinition::from_ron_file("assets/levels/prototype_quarantine_ward.ron")
        .unwrap_or_else(|error| panic!("failed to load prototype level: {error:?}"));
    level
        .validate()
        .unwrap_or_else(|error| panic!("invalid prototype level: {error:?}"));

    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.45, 0.85, 0.72), Vec2::new(34.0, 48.0)),
        Transform::from_xyz(level.apothecary_start.x, level.apothecary_start.y, 10.0),
        Apothecary,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.08, 0.10, 0.13), Vec2::new(900.0, 520.0)),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    for wall in &level.walls {
        spawn_wall(&mut commands, wall.center, wall.half_extents * 2.0);
    }

    for position in &level.contaminants {
        spawn_contaminant(&mut commands, *position);
    }

    for pickup in &level.pickups {
        spawn_pickup(&mut commands, pickup.position, pickup.kind.clone());
    }

    for door in &level.doors {
        spawn_door(
            &mut commands,
            door.position,
            door.half_extents * 2.0,
            door.clearance_id.clone(),
            door.starts_locked,
        );
    }

    for terminal in &level.terminals {
        spawn_terminal(
            &mut commands,
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
                required_objectives: exit.required_objectives.clone(),
            },
        ));
    }

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
}

fn spawn_wall(commands: &mut Commands, center: Vec2, size: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.23, 0.28, 0.32), size),
        Transform::from_xyz(center.x, center.y, -5.0),
        Wall {
            half_extents: size * 0.5,
        },
    ));
}

fn spawn_contaminant(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.78, 0.26, 0.42), Vec2::splat(36.0)),
        Transform::from_xyz(position.x, position.y, 15.0),
        Contaminant { health: 2 },
    ));
}

fn spawn_pickup(commands: &mut Commands, position: Vec2, kind: PickupKind) {
    let color = match kind {
        PickupKind::ReagentRounds(_) => Color::srgb(0.90, 0.86, 0.42),
        PickupKind::MedGel(_) => Color::srgb(0.28, 0.88, 0.64),
        PickupKind::BioSample => Color::srgb(0.72, 0.36, 0.90),
        PickupKind::SecurityKeycard(_) => Color::srgb(0.36, 0.62, 0.96),
    };

    commands.spawn((
        Sprite::from_color(color, Vec2::splat(18.0)),
        Transform::from_xyz(position.x, position.y, 6.0),
        Pickup { kind },
    ));
}

fn spawn_door(
    commands: &mut Commands,
    center: Vec2,
    size: Vec2,
    clearance_id: String,
    locked: bool,
) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.20, 0.58, 0.62), size),
        Transform::from_xyz(center.x, center.y, -3.0),
        Wall {
            half_extents: size * 0.5,
        },
        Door {
            clearance_id,
            locked,
        },
    ));
}

fn spawn_terminal(
    commands: &mut Commands,
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
        Terminal { kind, objective_id },
    ));
}
