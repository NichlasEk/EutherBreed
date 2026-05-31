use bevy::prelude::*;
use game_core::ApothecaryVitals as CoreApothecaryVitals;

#[derive(Resource)]
pub struct ApothecaryVitals(pub CoreApothecaryVitals);

#[derive(Resource)]
pub struct ContaminantSpawnTimer(pub Timer);
