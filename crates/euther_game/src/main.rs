mod components;
mod geometry;
mod resources;
mod setup;
mod systems;

use bevy::prelude::*;
use resources::{AccessInventory, ApothecaryVitals, ContaminantSpawnTimer};
use setup::setup;
use systems::{
    aim_apothecary, collect_pickups, fire_syringe_round, move_apothecary, move_contaminants,
    move_projectiles, quit_on_escape, report_exit_overlap, resolve_contaminant_contact,
    resolve_projectile_hits, spawn_contaminants, unlock_doors, update_status_text,
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
                report_exit_overlap,
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
