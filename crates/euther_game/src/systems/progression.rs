use bevy::prelude::*;

use crate::components::LevelEntity;
use crate::resources::{CampaignRuntime, CampaignSignal, LevelRuntime};
use crate::setup::{load_level_from_campaign, spawn_level};

pub fn update_campaign_progress(
    mut commands: Commands,
    mut signal: ResMut<CampaignSignal>,
    mut runtime: ResMut<CampaignRuntime>,
    mut level_runtime: ResMut<LevelRuntime>,
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

    let level = load_level_from_campaign(&runtime, runtime.progress.current_level());
    spawn_level(&mut commands, &level);
    level_runtime.loaded_level_id = Some(runtime.progress.current_level().to_string());
}
