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

    pub fn contains_level(&self, level_id: &str) -> bool {
        self.levels.iter().any(|level| level.id == level_id)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignProgress {
    current_level: String,
}

impl CampaignProgress {
    pub fn start(campaign: &CampaignDefinition) -> Result<Self, CampaignValidationError> {
        campaign.validate()?;

        Ok(Self {
            current_level: campaign.start_level.clone(),
        })
    }

    pub fn current_level(&self) -> &str {
        &self.current_level
    }

    pub fn travel_to(
        &mut self,
        campaign: &CampaignDefinition,
        level_id: &str,
    ) -> Result<bool, CampaignTravelError> {
        self.travel_to_known_level(campaign.contains_level(level_id), level_id)
    }

    pub fn travel_to_known_level(
        &mut self,
        is_known_level: bool,
        level_id: &str,
    ) -> Result<bool, CampaignTravelError> {
        if !is_known_level {
            return Err(CampaignTravelError::UnknownLevel);
        }

        if self.current_level == level_id {
            return Ok(false);
        }

        self.current_level = level_id.to_string();
        Ok(true)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignTravelError {
    UnknownLevel,
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
        let quarantine =
            LevelDefinition::from_ron_file("../../assets/levels/prototype_quarantine_ward.ron")
                .expect("prototype level should load");
        let corridor =
            LevelDefinition::from_ron_file("../../assets/levels/lab_access_corridor.ron")
                .expect("corridor level should load");

        assert_eq!(
            campaign.validate_level_routes([&quarantine, &corridor]),
            Ok(())
        );
    }

    #[test]
    fn campaign_progress_starts_at_start_level() {
        let campaign = campaign();
        let progress = CampaignProgress::start(&campaign).expect("campaign should start");

        assert_eq!(progress.current_level(), "prototype_quarantine_ward");
    }

    #[test]
    fn campaign_progress_travels_to_known_level() {
        let campaign = campaign();
        let mut progress = CampaignProgress::start(&campaign).expect("campaign should start");

        assert_eq!(
            progress.travel_to(&campaign, "lab_access_corridor"),
            Ok(true)
        );
        assert_eq!(progress.current_level(), "lab_access_corridor");
    }

    #[test]
    fn campaign_progress_rejects_unknown_level() {
        let campaign = campaign();
        let mut progress = CampaignProgress::start(&campaign).expect("campaign should start");

        assert_eq!(
            progress.travel_to(&campaign, "unknown"),
            Err(CampaignTravelError::UnknownLevel)
        );
    }
}
