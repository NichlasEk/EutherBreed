mod components;
mod geometry;
mod resources;
mod setup;
mod systems;

use bevy::prelude::*;
use resources::{
    AccessInventory, ApothecaryVitals, CampaignRuntime, CampaignSignal, ContaminantSpawnTimer,
    LevelRuntime, ObjectiveState,
};
use setup::setup;
use systems::{
    aim_apothecary, collect_pickups, fire_syringe_round, interact_with_terminals, move_apothecary,
    move_contaminants, move_projectiles, quit_on_escape, report_exit_overlap,
    resolve_contaminant_contact, resolve_projectile_hits, spawn_contaminants, unlock_doors,
    update_campaign_progress, update_status_text,
};

const CONTAMINANT_SPAWN_SECONDS: f32 = 1.7;

fn main() {
    if std::env::args().any(|arg| arg == "--headless-smoke") {
        run_headless_smoke();
        return;
    }

    run_game();
}

fn run_game() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.015, 0.018, 0.025)))
        .insert_resource(initial_vitals())
        .insert_resource(initial_contaminant_timer())
        .insert_resource(AccessInventory::default())
        .insert_resource(ObjectiveState::default())
        .insert_resource(CampaignSignal::default())
        .insert_resource(initial_campaign_runtime())
        .insert_resource(initial_level_runtime())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "EutherBreed Prototype".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_apothecary,
                aim_apothecary,
                fire_syringe_round,
                move_projectiles,
                spawn_contaminants,
                move_contaminants,
                resolve_projectile_hits,
                resolve_contaminant_contact,
                collect_pickups,
                unlock_doors,
                interact_with_terminals,
                report_exit_overlap,
                update_campaign_progress,
                update_status_text,
                quit_on_escape,
            ),
        )
        .run();
}

fn run_headless_smoke() {
    let mut app = App::new();
    app.insert_resource(initial_vitals())
        .insert_resource(initial_contaminant_timer())
        .insert_resource(AccessInventory::default())
        .insert_resource(ObjectiveState::default())
        .insert_resource(CampaignSignal::default())
        .insert_resource(initial_campaign_runtime())
        .insert_resource(initial_level_runtime())
        .add_plugins(MinimalPlugins);

    app.update();

    let vitals = app.world().resource::<ApothecaryVitals>();
    println!(
        "headless smoke ok: health={} ammo={} bio_samples={}",
        vitals.0.health, vitals.0.ammo, vitals.0.bio_samples
    );
}

fn initial_vitals() -> ApothecaryVitals {
    ApothecaryVitals(game_core::ApothecaryVitals::new(100, 48, 0))
}

fn initial_contaminant_timer() -> ContaminantSpawnTimer {
    ContaminantSpawnTimer(Timer::from_seconds(
        CONTAMINANT_SPAWN_SECONDS,
        TimerMode::Repeating,
    ))
}

fn initial_campaign_runtime() -> CampaignRuntime {
    let definition = game_core::CampaignDefinition::from_ron_file("assets/campaigns/prototype.ron")
        .unwrap_or_else(|error| panic!("failed to load prototype campaign: {error:?}"));
    definition
        .load_and_validate_levels()
        .unwrap_or_else(|error| panic!("invalid prototype campaign content: {error:?}"));
    let progress = game_core::CampaignProgress::start(&definition)
        .unwrap_or_else(|error| panic!("invalid prototype campaign: {error:?}"));

    CampaignRuntime {
        definition,
        progress,
    }
}

fn initial_level_runtime() -> LevelRuntime {
    LevelRuntime {
        loaded_level_id: None,
    }
}
