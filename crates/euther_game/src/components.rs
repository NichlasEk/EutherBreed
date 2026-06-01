use bevy::prelude::*;

#[derive(Component)]
pub struct Apothecary;

#[derive(Component)]
pub struct ApothecaryAnimation {
    pub frames: Vec<Handle<Image>>,
    pub phase: f32,
    pub visual_offset: Vec2,
}

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub struct MapOverlay;

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct EffectLifetime(pub Timer);

#[derive(Component)]
pub struct Contaminant {
    pub id: Option<String>,
    pub health: i32,
    pub hit_flash: Timer,
}

#[derive(Component)]
pub struct ContaminantAnimation {
    pub base_image: Handle<Image>,
    pub stride_image: Handle<Image>,
    pub phase: f32,
}

#[derive(Component)]
pub struct Wall {
    pub half_extents: Vec2,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HudGaugeKind {
    Health,
    Ammo,
}

#[derive(Component)]
pub struct HudGaugePip {
    pub kind: HudGaugeKind,
    pub index: usize,
}

#[derive(Component)]
pub struct KeysText;

#[derive(Component)]
pub struct BioText;

#[derive(Component)]
pub struct NoticeText;

#[derive(Component)]
pub struct SectionText;

#[derive(Component)]
pub struct ObjectiveText;

#[derive(Component)]
pub struct PromptText;

#[derive(Component)]
pub struct Pickup {
    pub id: String,
    pub kind: game_core::PickupKind,
}

#[derive(Component)]
pub struct ExitZone {
    pub target: String,
    pub entry_id: String,
    pub required_objectives: Vec<String>,
    pub half_extents: Vec2,
}

#[derive(Component)]
pub struct Door {
    pub id: String,
    pub clearance_id: String,
    pub locked: bool,
    pub kind: game_core::DoorKind,
}

#[derive(Component)]
pub struct Terminal {
    pub id: String,
    pub kind: game_core::TerminalKind,
    pub objective_id: Option<String>,
}
