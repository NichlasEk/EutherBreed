use serde::{Deserialize, Serialize};

use crate::{LevelState, RunState};

pub const SAVE_GAME_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SaveGame {
    pub version: u32,
    pub run_state: RunState,
    pub level_state: LevelState,
}

impl SaveGame {
    pub fn new(run_state: RunState, level_state: LevelState) -> Self {
        Self {
            version: SAVE_GAME_VERSION,
            run_state,
            level_state,
        }
    }

    pub fn to_ron_string(&self) -> Result<String, SaveLoadError> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(SaveLoadError::Serialize)
    }

    pub fn from_ron_str(content: &str) -> Result<Self, SaveLoadError> {
        let save: Self = ron::from_str(content).map_err(SaveLoadError::Deserialize)?;

        if save.version != SAVE_GAME_VERSION {
            return Err(SaveLoadError::UnsupportedVersion(save.version));
        }

        Ok(save)
    }
}

#[derive(Debug)]
pub enum SaveLoadError {
    Serialize(ron::Error),
    Deserialize(ron::error::SpannedError),
    UnsupportedVersion(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApothecaryVitals, LevelState, RunState};

    fn save_game() -> SaveGame {
        let mut level_state = LevelState::default();
        level_state.grant_clearance("quarantine_green");
        level_state.complete_objective("analyze_contaminant_sample");

        SaveGame::new(
            RunState::new(
                ApothecaryVitals::new(88, 19, 2),
                "prototype_quarantine_ward",
            ),
            level_state,
        )
    }

    #[test]
    fn save_game_uses_current_version() {
        assert_eq!(save_game().version, SAVE_GAME_VERSION);
    }

    #[test]
    fn save_game_roundtrips_through_ron() {
        let save = save_game();
        let content = save.to_ron_string().expect("save should serialize");
        let loaded = SaveGame::from_ron_str(&content).expect("save should deserialize");

        assert_eq!(loaded, save);
    }

    #[test]
    fn unsupported_save_version_is_rejected() {
        let mut save = save_game();
        save.version = SAVE_GAME_VERSION + 1;
        let content = save.to_ron_string().expect("save should serialize");

        assert!(matches!(
            SaveGame::from_ron_str(&content),
            Err(SaveLoadError::UnsupportedVersion(_))
        ));
    }
}
