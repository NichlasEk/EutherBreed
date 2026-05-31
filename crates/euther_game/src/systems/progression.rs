use bevy::prelude::*;

use crate::components::LevelEntity;
use crate::resources::{
    ApothecaryVitals, CampaignRuntime, CampaignSignal, ContaminantSpawnTimer, CurrentLevelMap,
    GameNotice, LevelRuntime, LocalLevelState, PersistentLevelStates, SaveSlot,
};
use crate::setup::{load_level_from_campaign, spawn_level, update_level_runtime};
use crate::systems::save::{build_runtime_save, write_runtime_save};

const RESTART_HEALTH: i32 = 100;
const RESTART_AMMO: i32 = 48;

pub fn update_campaign_progress(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut signal: ResMut<CampaignSignal>,
    mut runtime: ResMut<CampaignRuntime>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut current_level_map: ResMut<CurrentLevelMap>,
    mut level_state: ResMut<LocalLevelState>,
    mut persistent_level_states: ResMut<PersistentLevelStates>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    vitals: Res<ApothecaryVitals>,
    save_slot: Res<SaveSlot>,
    mut notice: ResMut<GameNotice>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    let Some(pending_exit) = signal.pending_exit.take() else {
        return;
    };

    let target = pending_exit.target;
    let is_known_level = runtime.definition.contains_level(&target);
    let previous_level = runtime.progress.current_level().to_string();

    match runtime
        .progress
        .travel_to_known_level(is_known_level, &target)
    {
        Ok(true) => info!(
            "campaign traveled to level {}",
            runtime.progress.current_level()
        ),
        Ok(false) => {
            debug!("campaign already at level {}", target);
            return;
        }
        Err(error) => {
            warn!("campaign travel to {} failed: {:?}", target, error);
            return;
        }
    }

    if level_runtime.loaded_level_id.as_deref() == Some(runtime.progress.current_level()) {
        return;
    }

    persistent_level_states
        .0
        .insert(previous_level, level_state.0.clone());

    for entity in &level_entities {
        commands.entity(entity).despawn();
    }

    load_level_local_state(
        &mut level_state,
        &persistent_level_states,
        runtime.progress.current_level(),
    );
    contaminant_timer.0.reset();

    let level = load_level_from_campaign(&runtime, runtime.progress.current_level());
    level_runtime.pending_entry_id = Some(pending_exit.entry_id);
    spawn_level(
        &mut commands,
        &asset_server,
        &level,
        &level_state.0,
        level_runtime.pending_entry_id.as_deref(),
        None,
    );
    level_runtime.loaded_level_id = Some(runtime.progress.current_level().to_string());
    update_level_runtime(
        &mut level_runtime,
        &mut current_level_map,
        &level,
        &mut contaminant_timer,
    );

    let run_position =
        crate::setup::apothecary_spawn_position(&level, level_runtime.pending_entry_id.as_deref());
    let save = build_runtime_save(
        &vitals,
        &runtime,
        &level_state,
        &persistent_level_states,
        run_position,
    );
    match write_runtime_save(&save_slot.path, &save) {
        Ok(()) => {
            notice.show("Autosaved", 1.4);
            info!("autosave written to {}", save_slot.path.display());
        }
        Err(error) => {
            notice.show("Autosave failed", 2.0);
            warn!(
                "autosave to {} failed: {:?}",
                save_slot.path.display(),
                error
            );
        }
    }
}

pub fn restart_current_level_on_death(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut vitals: ResMut<ApothecaryVitals>,
    runtime: Res<CampaignRuntime>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut current_level_map: ResMut<CurrentLevelMap>,
    mut level_state: ResMut<LocalLevelState>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    mut notice: ResMut<GameNotice>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    if vitals.0.health > 0 {
        return;
    }

    if !input.just_pressed(KeyCode::KeyR) {
        notice.show("Suit breached - press R to restart section", 0.4);
        return;
    }

    for entity in &level_entities {
        commands.entity(entity).despawn();
    }

    vitals.0 = game_core::ApothecaryVitals::new(RESTART_HEALTH, RESTART_AMMO, 0);
    level_state.0 = game_core::LevelState::default();
    contaminant_timer.0.reset();

    let level = load_level_from_campaign(&runtime, runtime.progress.current_level());
    spawn_level(
        &mut commands,
        &asset_server,
        &level,
        &level_state.0,
        None,
        None,
    );
    level_runtime.loaded_level_id = Some(runtime.progress.current_level().to_string());
    level_runtime.pending_entry_id = None;
    update_level_runtime(
        &mut level_runtime,
        &mut current_level_map,
        &level,
        &mut contaminant_timer,
    );

    notice.show("Section restarted", 1.4);
}

fn load_level_local_state(
    level_state: &mut LocalLevelState,
    persistent_level_states: &PersistentLevelStates,
    level_id: &str,
) {
    level_state.0 = persistent_level_states
        .0
        .get(level_id)
        .cloned()
        .unwrap_or_default();
}
