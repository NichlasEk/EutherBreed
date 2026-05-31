use std::fmt;
use std::path::{Path, PathBuf};

use bevy::prelude::*;

use crate::components::{Apothecary, LevelEntity};
use crate::resources::{
    ApothecaryVitals, CampaignRuntime, ContaminantSpawnTimer, GameNotice, LevelRuntime,
    LocalLevelState, PersistentLevelStates, SaveSlot,
};
use crate::setup::{load_level_from_campaign, spawn_level, update_level_runtime};

pub fn quick_save_on_key(
    input: Res<ButtonInput<KeyCode>>,
    save_slot: Res<SaveSlot>,
    vitals: Res<ApothecaryVitals>,
    campaign_runtime: Res<CampaignRuntime>,
    level_state: Res<LocalLevelState>,
    persistent_level_states: Res<PersistentLevelStates>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    mut notice: ResMut<GameNotice>,
) {
    if !input.just_pressed(KeyCode::F5) {
        return;
    }

    let save = build_runtime_save(
        &vitals,
        &campaign_runtime,
        &level_state,
        &persistent_level_states,
        apothecary_query.translation.xy(),
    );

    match write_runtime_save(&save_slot.path, &save) {
        Ok(()) => {
            notice.show("Saved", 1.4);
            info!("quick save written to {}", save_slot.path.display());
        }
        Err(error) => {
            notice.show("Save failed", 2.0);
            warn!(
                "quick save to {} failed: {:?}",
                save_slot.path.display(),
                error
            );
        }
    }
}

pub fn quick_load_on_key(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    save_slot: Res<SaveSlot>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut campaign_runtime: ResMut<CampaignRuntime>,
    mut level_state: ResMut<LocalLevelState>,
    mut persistent_level_states: ResMut<PersistentLevelStates>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut notice: ResMut<GameNotice>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    if !input.just_pressed(KeyCode::F9) {
        return;
    }

    let save = match game_core::SaveGame::read_from_file(&save_slot.path) {
        Ok(save) => save,
        Err(error) => {
            notice.show("No save found", 2.0);
            warn!(
                "quick load from {} failed: {:?}",
                save_slot.path.display(),
                error
            );
            return;
        }
    };

    apply_save_to_runtime(
        &save,
        &mut vitals,
        &mut campaign_runtime,
        &mut level_state,
        &mut persistent_level_states,
    )
    .unwrap_or_else(|error| {
        panic!(
            "failed to apply save level {}: {error:?}",
            save.run_state.current_level
        )
    });

    for entity in &level_entities {
        commands.entity(entity).despawn();
    }

    contaminant_timer.0.reset();

    let level =
        load_level_from_campaign(&campaign_runtime, campaign_runtime.progress.current_level());
    level_runtime.pending_entry_id = None;
    spawn_level(
        &mut commands,
        &level,
        &level_state.0,
        None,
        Some(save.run_state.position),
    );
    level_runtime.loaded_level_id = Some(campaign_runtime.progress.current_level().to_string());
    update_level_runtime(&mut level_runtime, &level, &mut contaminant_timer);

    notice.show("Loaded", 1.4);
    info!("quick load read from {}", save_slot.path.display());
}

pub fn build_runtime_save(
    vitals: &ApothecaryVitals,
    campaign_runtime: &CampaignRuntime,
    level_state: &LocalLevelState,
    persistent_level_states: &PersistentLevelStates,
    position: Vec2,
) -> game_core::SaveGame {
    let mut level_states = persistent_level_states.0.clone();
    level_states.insert(
        campaign_runtime.progress.current_level().to_string(),
        level_state.0.clone(),
    );

    game_core::SaveGame::with_level_states(
        game_core::RunState::new_at(
            vitals.0.clone(),
            campaign_runtime.progress.current_level().to_string(),
            position,
        ),
        level_states,
    )
}

pub fn apply_save_to_runtime(
    save: &game_core::SaveGame,
    vitals: &mut ApothecaryVitals,
    campaign_runtime: &mut CampaignRuntime,
    level_state: &mut LocalLevelState,
    persistent_level_states: &mut PersistentLevelStates,
) -> Result<(), game_core::CampaignTravelError> {
    campaign_runtime
        .progress
        .travel_to(&campaign_runtime.definition, &save.run_state.current_level)?;

    vitals.0 = save.run_state.vitals.clone();
    persistent_level_states.0 = save.level_states.clone();
    level_state.0 = save.current_level_state();

    Ok(())
}

pub fn write_runtime_save(
    path: impl AsRef<Path>,
    save: &game_core::SaveGame,
) -> Result<(), RuntimeSaveError> {
    let path = path.as_ref();

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(RuntimeSaveError::CreateDirectory)?;
    }

    save.write_to_file(path).map_err(RuntimeSaveError::Save)
}

#[derive(Debug)]
pub enum RuntimeSaveError {
    CreateDirectory(std::io::Error),
    Save(game_core::SaveLoadError),
}

impl fmt::Display for RuntimeSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateDirectory(error) => {
                write!(formatter, "failed to create directory: {error}")
            }
            Self::Save(error) => write!(formatter, "failed to write save: {error:?}"),
        }
    }
}

impl std::error::Error for RuntimeSaveError {}

pub fn default_save_path() -> PathBuf {
    PathBuf::from("saves/slot1.ron")
}
