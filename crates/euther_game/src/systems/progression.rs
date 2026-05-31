use bevy::prelude::*;

use crate::components::LevelEntity;
use crate::resources::{
    CampaignRuntime, CampaignSignal, ContaminantSpawnTimer, LevelRuntime, LocalLevelState,
};
use crate::setup::{load_level_from_campaign, spawn_level};

pub fn update_campaign_progress(
    mut commands: Commands,
    mut signal: ResMut<CampaignSignal>,
    mut runtime: ResMut<CampaignRuntime>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut level_state: ResMut<LocalLevelState>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    let Some(target) = signal.pending_exit_target.take() else {
        return;
    };

    let is_known_level = runtime.definition.contains_level(&target);

    match runtime
        .progress
        .travel_to_known_level(is_known_level, &target)
    {
        Ok(true) => info!(
            "campaign traveled to level {}",
            runtime.progress.current_level()
        ),
        Ok(false) => debug!("campaign already at level {}", target),
        Err(error) => warn!("campaign travel to {} failed: {:?}", target, error),
    }

    if level_runtime.loaded_level_id.as_deref() == Some(runtime.progress.current_level()) {
        return;
    }

    for entity in &level_entities {
        commands.entity(entity).despawn();
    }

    reset_level_local_state(&mut level_state, &mut contaminant_timer);

    let level = load_level_from_campaign(&runtime, runtime.progress.current_level());
    spawn_level(&mut commands, &level);
    level_runtime.loaded_level_id = Some(runtime.progress.current_level().to_string());
}

fn reset_level_local_state(
    level_state: &mut LocalLevelState,
    contaminant_timer: &mut ContaminantSpawnTimer,
) {
    level_state.0.reset_for_level_travel();
    contaminant_timer.0.reset();
}
