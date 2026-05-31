use bevy::prelude::*;
use game_core::ObjectiveProgress;
use game_core::{ApothecaryVitals as CoreApothecaryVitals, CampaignDefinition, CampaignProgress};
use std::collections::HashSet;

#[derive(Resource)]
pub struct ApothecaryVitals(pub CoreApothecaryVitals);

#[derive(Resource)]
pub struct ContaminantSpawnTimer(pub Timer);

#[derive(Resource, Default)]
pub struct AccessInventory {
    pub clearances: HashSet<String>,
}

#[derive(Resource, Default)]
pub struct ObjectiveState(pub ObjectiveProgress);

#[derive(Resource, Default)]
pub struct CampaignSignal {
    pub pending_exit_target: Option<String>,
}

#[derive(Resource)]
pub struct CampaignRuntime {
    pub definition: CampaignDefinition,
    pub progress: CampaignProgress,
}

#[derive(Resource)]
pub struct LevelRuntime {
    pub loaded_level_id: Option<String>,
}
