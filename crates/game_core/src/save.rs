use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{LevelState, RunState};

pub const SAVE_GAME_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SaveGame {
    pub version: u32,
    pub run_state: RunState,
    pub level_states: HashMap<String, LevelState>,
}

impl SaveGame {
    pub fn new(run_state: RunState, level_state: LevelState) -> Self {
        let mut level_states = HashMap::new();
        level_states.insert(run_state.current_level.clone(), level_state);

        Self {
            version: SAVE_GAME_VERSION,
            run_state,
            level_states,
        }
    }

    pub fn with_level_states(
        run_state: RunState,
        level_states: HashMap<String, LevelState>,
    ) -> Self {
        Self {
            version: SAVE_GAME_VERSION,
            run_state,
            level_states,
        }
    }

    pub fn current_level_state(&self) -> LevelState {
        self.level_state(&self.run_state.current_level)
    }

    pub fn level_state(&self, level_id: &str) -> LevelState {
        self.level_states.get(level_id).cloned().unwrap_or_default()
    }

    pub fn to_ron_string(&self) -> Result<String, SaveLoadError> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(SaveLoadError::Serialize)
    }

    pub fn from_ron_str(content: &str) -> Result<Self, SaveLoadError> {
        let save: Self = match ron::from_str(content) {
            Ok(save) => save,
            Err(error) => {
                return LegacySaveGame::from_ron_str(content)
                    .map(Self::from)
                    .map_err(|_| SaveLoadError::Deserialize(error));
            }
        };

        if save.version != SAVE_GAME_VERSION {
            return Err(SaveLoadError::UnsupportedVersion(save.version));
        }

        Ok(save)
    }

    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<(), SaveLoadError> {
        let content = self.to_ron_string()?;
        std::fs::write(path, content).map_err(SaveLoadError::Write)
    }

    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self, SaveLoadError> {
        let content = std::fs::read_to_string(path).map_err(SaveLoadError::Read)?;
        Self::from_ron_str(&content)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct LegacySaveGame {
    version: u32,
    run_state: RunState,
    level_state: LevelState,
}

impl LegacySaveGame {
    fn from_ron_str(content: &str) -> Result<Self, SaveLoadError> {
        let save: Self = ron::from_str(content).map_err(SaveLoadError::Deserialize)?;

        if save.version != 1 {
            return Err(SaveLoadError::UnsupportedVersion(save.version));
        }

        Ok(save)
    }
}

impl From<LegacySaveGame> for SaveGame {
    fn from(save: LegacySaveGame) -> Self {
        Self::new(save.run_state, save.level_state)
    }
}

#[derive(Debug)]
pub enum SaveLoadError {
    Read(std::io::Error),
    Write(std::io::Error),
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
    fn save_game_tracks_current_level_state_by_level_id() {
        let save = save_game();
        let loaded_state = save.current_level_state();

        assert!(loaded_state.has_clearance("quarantine_green"));
        assert!(
            loaded_state
                .objectives
                .is_complete("analyze_contaminant_sample")
        );
        assert_eq!(save.level_state("unknown"), LevelState::default());
    }

    #[test]
    fn save_game_can_store_multiple_level_states() {
        let mut quarantine_state = LevelState::default();
        quarantine_state.grant_clearance("quarantine_green");
        let mut lab_state = LevelState::default();
        lab_state.complete_objective("stabilize_lab");
        let mut level_states = std::collections::HashMap::new();
        level_states.insert("prototype_quarantine_ward".to_string(), quarantine_state);
        level_states.insert("lab_access_corridor".to_string(), lab_state);

        let save = SaveGame::with_level_states(
            RunState::new(ApothecaryVitals::new(71, 9, 4), "lab_access_corridor"),
            level_states,
        );

        assert_eq!(save.level_states.len(), 2);
        assert!(
            save.current_level_state()
                .objectives
                .is_complete("stabilize_lab")
        );
        assert!(
            save.level_state("prototype_quarantine_ward")
                .has_clearance("quarantine_green")
        );
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

    #[test]
    fn version_one_save_migrates_current_level_state() {
        let content = r#"(
            version: 1,
            run_state: (
                vitals: (health: 88, ammo: 19, bio_samples: 2),
                current_level: "prototype_quarantine_ward",
            ),
            level_state: (
                clearances: ["quarantine_green"],
                objectives: (
                    completed: ["analyze_contaminant_sample"],
                ),
            ),
        )"#;

        let save = SaveGame::from_ron_str(content).expect("legacy save should migrate");

        assert_eq!(save.version, SAVE_GAME_VERSION);
        assert_eq!(save.level_states.len(), 1);
        assert!(save.current_level_state().has_clearance("quarantine_green"));
    }

    #[test]
    fn save_game_roundtrips_through_file() {
        let save = save_game();
        let path =
            std::env::temp_dir().join(format!("euther_save_test_{}.ron", std::process::id()));

        save.write_to_file(&path).expect("save should write");
        let loaded = SaveGame::read_from_file(&path).expect("save should read");
        let _ = std::fs::remove_file(&path);

        assert_eq!(loaded, save);
    }
}
