use bevy::prelude::*;

#[derive(Component)]
pub struct Apothecary;

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct Contaminant {
    pub health: i32,
}

#[derive(Component)]
pub struct Wall {
    pub half_extents: Vec2,
}

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct Pickup {
    pub kind: game_core::PickupKind,
}

#[derive(Component)]
pub struct ExitZone {
    pub target: &'static str,
}
