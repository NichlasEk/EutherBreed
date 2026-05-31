use bevy::prelude::*;
use game_core::{
    ApothecaryVitals as CoreApothecaryVitals, CampaignDefinition, CampaignProgress, LevelState,
};

#[derive(Resource)]
pub struct ApothecaryVitals(pub CoreApothecaryVitals);

#[derive(Resource)]
pub struct ContaminantSpawnTimer(pub Timer);

#[derive(Resource, Default)]
pub struct LocalLevelState(pub LevelState);

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
