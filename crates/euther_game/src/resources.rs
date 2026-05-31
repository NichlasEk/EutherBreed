use bevy::prelude::*;
use game_core::LevelDefinition;
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
    pub camera_center: Vec2,
    pub camera_size: Vec2,
    pub dynamic_spawn_points: Vec<Vec2>,
    pub dynamic_spawn_cursor: usize,
    pub dynamic_spawn_interval_seconds: f32,
    pub available_exits: Vec<String>,
}

#[derive(Resource, Default)]
pub struct CurrentLevelMap {
    pub level: Option<LevelDefinition>,
}

#[derive(Resource)]
pub struct SaveSlot {
    pub path: PathBuf,
}

#[derive(Resource)]
pub struct GameNotice {
    pub text: String,
    pub timer: Timer,
}

impl Default for GameNotice {
    fn default() -> Self {
        Self {
            text: String::new(),
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

impl GameNotice {
    pub fn show(&mut self, text: impl Into<String>, seconds: f32) {
        self.text = text.into();
        self.timer = Timer::from_seconds(seconds, TimerMode::Once);
        self.timer.reset();
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn is_visible(&self) -> bool {
        !self.text.is_empty()
    }
}
