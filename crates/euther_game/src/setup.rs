use bevy::prelude::*;
use game_core::{LevelDefinition, PickupKind};

use crate::components::{Apothecary, Contaminant, ExitZone, Pickup, StatusText, Wall};

pub fn setup(mut commands: Commands) {
    let level = LevelDefinition::prototype_quarantine_ward();

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
        spawn_pickup(&mut commands, pickup.position, pickup.kind);
    }

    for exit in &level.exits {
        commands.spawn((
            Sprite::from_color(
                Color::srgba(0.30, 0.72, 0.95, 0.55),
                exit.half_extents * 2.0,
            ),
            Transform::from_xyz(exit.position.x, exit.position.y, 2.0),
            ExitZone {
                target: exit.target,
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
    };

    commands.spawn((
        Sprite::from_color(color, Vec2::splat(18.0)),
        Transform::from_xyz(position.x, position.y, 6.0),
        Pickup { kind },
    ));
}
