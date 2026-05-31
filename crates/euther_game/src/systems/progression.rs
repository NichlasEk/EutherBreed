use bevy::prelude::*;

use crate::components::LevelEntity;
use crate::resources::{
    ApothecaryVitals, CampaignRuntime, CampaignSignal, ContaminantSpawnTimer, LevelRuntime,
    LocalLevelState, PersistentLevelStates, SaveSlot,
};
use crate::setup::{load_level_from_campaign, spawn_level};
use crate::systems::save::{build_runtime_save, write_runtime_save};

pub fn update_campaign_progress(
    mut commands: Commands,
    mut signal: ResMut<CampaignSignal>,
    mut runtime: ResMut<CampaignRuntime>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut level_state: ResMut<LocalLevelState>,
    mut persistent_level_states: ResMut<PersistentLevelStates>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    vitals: Res<ApothecaryVitals>,
    save_slot: Res<SaveSlot>,
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
        &level,
        &level_state.0,
        level_runtime.pending_entry_id.as_deref(),
    );
    level_runtime.loaded_level_id = Some(runtime.progress.current_level().to_string());

    let save = build_runtime_save(&vitals, &runtime, &level_state, &persistent_level_states);
    match write_runtime_save(&save_slot.path, &save) {
        Ok(()) => info!("autosave written to {}", save_slot.path.display()),
        Err(error) => warn!(
            "autosave to {} failed: {:?}",
            save_slot.path.display(),
            error
        ),
    }
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
