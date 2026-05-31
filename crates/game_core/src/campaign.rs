use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::LevelDefinition;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CampaignDefinition {
    pub name: String,
    pub start_level: String,
    pub levels: Vec<CampaignLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CampaignLevel {
    pub id: String,
    pub path: String,
}

impl CampaignDefinition {
    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self, CampaignLoadError> {
        let content = fs::read_to_string(path).map_err(CampaignLoadError::Read)?;
        ron::from_str(&content).map_err(CampaignLoadError::Parse)
    }

    pub fn validate(&self) -> Result<(), CampaignValidationError> {
        if self.name.trim().is_empty() {
            return Err(CampaignValidationError::MissingName);
        }

        if self.start_level.trim().is_empty() {
            return Err(CampaignValidationError::MissingStartLevel);
        }

        let ids = self.level_ids()?;

        if !ids.contains(&self.start_level) {
            return Err(CampaignValidationError::UnknownStartLevel);
        }

        Ok(())
    }

    pub fn validate_level_routes<'a>(
        &self,
        levels: impl IntoIterator<Item = &'a LevelDefinition>,
    ) -> Result<(), CampaignValidationError> {
        self.validate()?;

        let ids = self.level_ids()?;

        for level in levels {
            for exit in &level.exits {
                if !ids.contains(&exit.target) {
                    return Err(CampaignValidationError::UnknownExitTarget);
                }
            }
        }

        Ok(())
    }

    fn level_ids(&self) -> Result<HashSet<String>, CampaignValidationError> {
        if self.levels.is_empty() {
            return Err(CampaignValidationError::NoLevels);
        }

        let mut ids = HashSet::new();

        for level in &self.levels {
            if level.id.trim().is_empty() || level.path.trim().is_empty() {
                return Err(CampaignValidationError::InvalidLevelEntry);
            }

            if !ids.insert(level.id.clone()) {
                return Err(CampaignValidationError::DuplicateLevelId);
            }
        }

        Ok(ids)
    }
}

#[derive(Debug)]
pub enum CampaignLoadError {
    Read(std::io::Error),
    Parse(ron::error::SpannedError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignValidationError {
    MissingName,
    MissingStartLevel,
    NoLevels,
    InvalidLevelEntry,
    DuplicateLevelId,
    UnknownStartLevel,
    UnknownExitTarget,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn campaign() -> CampaignDefinition {
        CampaignDefinition {
            name: "Prototype Campaign".to_string(),
            start_level: "prototype_quarantine_ward".to_string(),
            levels: vec![
                CampaignLevel {
                    id: "prototype_quarantine_ward".to_string(),
                    path: "assets/levels/prototype_quarantine_ward.ron".to_string(),
                },
                CampaignLevel {
                    id: "lab_access_corridor".to_string(),
                    path: "assets/levels/lab_access_corridor.ron".to_string(),
                },
            ],
        }
    }

    #[test]
    fn campaign_validates_when_start_level_exists() {
        assert_eq!(campaign().validate(), Ok(()));
    }

    #[test]
    fn campaign_rejects_duplicate_level_ids() {
        let mut campaign = campaign();
        campaign.levels.push(CampaignLevel {
            id: "lab_access_corridor".to_string(),
            path: "other.ron".to_string(),
        });

        assert_eq!(
            campaign.validate(),
            Err(CampaignValidationError::DuplicateLevelId)
        );
    }

    #[test]
    fn campaign_rejects_unknown_exit_targets() {
        let mut campaign = campaign();
        campaign
            .levels
            .retain(|level| level.id != "lab_access_corridor");
        let level = LevelDefinition::prototype_quarantine_ward();

        assert_eq!(
            campaign.validate_level_routes([&level]),
            Err(CampaignValidationError::UnknownExitTarget)
        );
    }

    #[test]
    fn campaign_file_loads_and_validates_routes() {
        let campaign = CampaignDefinition::from_ron_file("../../assets/campaigns/prototype.ron")
            .expect("prototype campaign should load");
        let level =
            LevelDefinition::from_ron_file("../../assets/levels/prototype_quarantine_ward.ron")
                .expect("prototype level should load");

        assert_eq!(campaign.validate_level_routes([&level]), Ok(()));
    }
}
