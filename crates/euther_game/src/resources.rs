use bevy::prelude::*;
use game_core::{
    ApothecaryVitals as CoreApothecaryVitals, CampaignDefinition, CampaignProgress, LevelState,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Resource)]
pub struct ApothecaryVitals(pub CoreApothecaryVitals);

#[derive(Resource)]
pub struct ContaminantSpawnTimer(pub Timer);

#[derive(Resource, Default)]
pub struct LocalLevelState(pub LevelState);

#[derive(Resource, Default)]
pub struct PersistentLevelStates(pub HashMap<String, LevelState>);

#[derive(Resource, Default)]
pub struct CampaignSignal {
    pub pending_exit: Option<PendingExit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingExit {
    pub target: String,
    pub entry_id: String,
}

#[derive(Resource)]
pub struct CampaignRuntime {
    pub definition: CampaignDefinition,
    pub progress: CampaignProgress,
}

#[derive(Resource)]
pub struct LevelRuntime {
    pub loaded_level_id: Option<String>,
    pub pending_entry_id: Option<String>,
}

#[derive(Resource)]
pub struct SaveSlot {
    pub path: PathBuf,
}
