use bevy::prelude::*;
use game_core::ApothecaryVitals as CoreApothecaryVitals;
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
pub struct ObjectiveState {
    pub completed: HashSet<String>,
}

#[derive(Resource, Default)]
pub struct CampaignSignal {
    pub pending_exit_target: Option<String>,
}
