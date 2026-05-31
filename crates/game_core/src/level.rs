use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::AxisAlignedBox;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LevelDefinition {
    pub name: String,
    pub bounds: AxisAlignedBox,
    pub apothecary_start: Vec2,
    pub walls: Vec<AxisAlignedBox>,
    pub contaminants: Vec<ContaminantDefinition>,
    pub pickups: Vec<PrototypeEntity<PickupKind>>,
    pub doors: Vec<DoorDefinition>,
    pub terminals: Vec<TerminalDefinition>,
    pub objectives: Vec<ObjectiveDefinition>,
    pub entry_points: Vec<LevelEntryPoint>,
    pub exits: Vec<LevelExit>,
    #[serde(default)]
    pub spawn_points: Vec<Vec2>,
    #[serde(default)]
    pub spawn_interval_seconds: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PrototypeEntity<T> {
    pub id: String,
    pub position: Vec2,
    pub kind: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum PickupKind {
    ReagentRounds(i32),
    MedGel(i32),
    BioSample,
    SecurityKeycard(String),
    AreaScan,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ContaminantDefinition {
    pub id: String,
    pub position: Vec2,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LevelExit {
    pub position: Vec2,
    pub half_extents: Vec2,
    pub target: String,
    pub entry_id: String,
    pub required_objectives: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LevelEntryPoint {
    pub id: String,
    pub position: Vec2,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DoorDefinition {
    pub id: String,
    pub position: Vec2,
    pub half_extents: Vec2,
    pub clearance_id: String,
    pub starts_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TerminalDefinition {
    pub id: String,
    pub position: Vec2,
    pub kind: TerminalKind,
    pub objective_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TerminalKind {
    LabAnalyzer,
    ShipLog,
    SupplyConsole,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ObjectiveDefinition {
    pub id: String,
    pub label: String,
    pub required: bool,
}

impl LevelDefinition {
    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self, LevelLoadError> {
        let content = fs::read_to_string(path).map_err(LevelLoadError::Read)?;
        ron::from_str(&content).map_err(LevelLoadError::Parse)
    }

    pub fn validate(&self) -> Result<(), LevelValidationError> {
        if self.name.trim().is_empty() {
            return Err(LevelValidationError::MissingName);
        }

        if self.walls.is_empty() {
            return Err(LevelValidationError::NoWalls);
        }

        if self.exits.is_empty() {
            return Err(LevelValidationError::NoExits);
        }

        if !point_inside_box(self.apothecary_start, self.bounds) {
            return Err(LevelValidationError::StartOutsideBounds);
        }

        let mut contaminant_ids = HashSet::new();
        for contaminant in &self.contaminants {
            if contaminant.id.trim().is_empty() || !contaminant_ids.insert(contaminant.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }
        }

        let mut pickup_ids = HashSet::new();
        for pickup in &self.pickups {
            if pickup.id.trim().is_empty() || !pickup_ids.insert(pickup.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            match pickup.kind {
                PickupKind::ReagentRounds(amount) | PickupKind::MedGel(amount) if amount <= 0 => {
                    return Err(LevelValidationError::InvalidPickupAmount);
                }
                PickupKind::SecurityKeycard(ref clearance_id) if clearance_id.trim().is_empty() => {
                    return Err(LevelValidationError::InvalidClearanceId);
                }
                _ => {}
            }
        }

        let mut door_ids = HashSet::new();
        for door in &self.doors {
            if door.id.trim().is_empty() || !door_ids.insert(door.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            if door.clearance_id.trim().is_empty() {
                return Err(LevelValidationError::InvalidClearanceId);
            }
        }

        let mut terminal_ids = HashSet::new();
        for terminal in &self.terminals {
            if terminal.id.trim().is_empty() || !terminal_ids.insert(terminal.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            if matches!(terminal.objective_id, Some(ref objective_id) if objective_id.trim().is_empty())
            {
                return Err(LevelValidationError::InvalidObjective);
            }
        }

        for objective in &self.objectives {
            if objective.id.trim().is_empty() || objective.label.trim().is_empty() {
                return Err(LevelValidationError::InvalidObjective);
            }
        }

        let mut entry_ids = HashSet::new();
        for entry_point in &self.entry_points {
            if entry_point.id.trim().is_empty() || !entry_ids.insert(entry_point.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            if !point_inside_box(entry_point.position, self.bounds) {
                return Err(LevelValidationError::EntryOutsideBounds);
            }
        }

        for exit in &self.exits {
            if exit.target.trim().is_empty() || exit.entry_id.trim().is_empty() {
                return Err(LevelValidationError::InvalidExit);
            }

            if exit
                .required_objectives
                .iter()
                .any(|objective_id| objective_id.trim().is_empty())
            {
                return Err(LevelValidationError::InvalidObjective);
            }
        }

        if matches!(self.spawn_interval_seconds, Some(seconds) if seconds <= 0.0) {
            return Err(LevelValidationError::InvalidSpawnInterval);
        }

        Ok(())
    }

    pub fn prototype_quarantine_ward() -> Self {
        Self {
            name: "prototype_quarantine_ward".to_string(),
            bounds: AxisAlignedBox::new(Vec2::ZERO, Vec2::new(450.0, 260.0)),
            apothecary_start: Vec2::new(-300.0, -170.0),
            walls: vec![
                wall(0.0, 274.0, 920.0, 28.0),
                wall(0.0, -274.0, 920.0, 28.0),
                wall(-474.0, 0.0, 28.0, 548.0),
                wall(474.0, 0.0, 28.0, 548.0),
                wall(0.0, 82.0, 230.0, 26.0),
                wall(-180.0, -110.0, 180.0, 24.0),
                wall(220.0, -64.0, 26.0, 180.0),
            ],
            contaminants: vec![
                ContaminantDefinition {
                    id: "ward_contaminant_alpha".to_string(),
                    position: Vec2::new(320.0, 180.0),
                },
                ContaminantDefinition {
                    id: "ward_contaminant_beta".to_string(),
                    position: Vec2::new(380.0, -185.0),
                },
            ],
            pickups: vec![
                PrototypeEntity {
                    id: "ward_rounds_a".to_string(),
                    position: Vec2::new(-70.0, -190.0),
                    kind: PickupKind::ReagentRounds(12),
                },
                PrototypeEntity {
                    id: "ward_medgel_a".to_string(),
                    position: Vec2::new(310.0, 80.0),
                    kind: PickupKind::MedGel(25),
                },
                PrototypeEntity {
                    id: "ward_quarantine_green_keycard".to_string(),
                    position: Vec2::new(-310.0, 165.0),
                    kind: PickupKind::SecurityKeycard("quarantine_green".to_string()),
                },
            ],
            doors: vec![DoorDefinition {
                id: "ward_quarantine_green_door".to_string(),
                position: Vec2::new(0.0, 82.0),
                half_extents: Vec2::new(32.0, 13.0),
                clearance_id: "quarantine_green".to_string(),
                starts_locked: true,
            }],
            terminals: vec![TerminalDefinition {
                id: "ward_lab_analyzer".to_string(),
                position: Vec2::new(360.0, -96.0),
                kind: TerminalKind::LabAnalyzer,
                objective_id: Some("analyze_contaminant_sample".to_string()),
            }],
            objectives: vec![ObjectiveDefinition {
                id: "analyze_contaminant_sample".to_string(),
                label: "Analyze contaminant sample".to_string(),
                required: true,
            }],
            entry_points: vec![LevelEntryPoint {
                id: "from_lab_access_corridor".to_string(),
                position: Vec2::new(390.0, 0.0),
            }],
            exits: vec![LevelExit {
                position: Vec2::new(432.0, 205.0),
                half_extents: Vec2::new(22.0, 46.0),
                target: "lab_access_corridor".to_string(),
                entry_id: "from_quarantine_ward".to_string(),
                required_objectives: vec!["analyze_contaminant_sample".to_string()],
            }],
            spawn_points: vec![Vec2::new(-405.0, 205.0), Vec2::new(405.0, -205.0)],
            spawn_interval_seconds: Some(4.5),
        }
    }
}

#[derive(Debug)]
pub enum LevelLoadError {
    Read(std::io::Error),
    Parse(ron::error::SpannedError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelValidationError {
    MissingName,
    NoWalls,
    NoExits,
    StartOutsideBounds,
    EntryOutsideBounds,
    InvalidPickupAmount,
    InvalidEntityId,
    InvalidClearanceId,
    InvalidObjective,
    InvalidExit,
    InvalidSpawnInterval,
}

const fn wall(x: f32, y: f32, width: f32, height: f32) -> AxisAlignedBox {
    AxisAlignedBox::new(Vec2::new(x, y), Vec2::new(width * 0.5, height * 0.5))
}

fn point_inside_box(point: Vec2, area: AxisAlignedBox) -> bool {
    let min = area.center - area.half_extents;
    let max = area.center + area.half_extents;

    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prototype_level_has_required_starting_parts() {
        let level = LevelDefinition::prototype_quarantine_ward();

        assert!(!level.name.is_empty());
        assert!(!level.walls.is_empty());
        assert!(!level.contaminants.is_empty());
        assert!(!level.pickups.is_empty());
        assert!(!level.doors.is_empty());
        assert!(!level.terminals.is_empty());
        assert!(!level.objectives.is_empty());
        assert!(!level.entry_points.is_empty());
        assert!(!level.exits.is_empty());
    }

    #[test]
    fn apothecary_starts_inside_bounds() {
        let level = LevelDefinition::prototype_quarantine_ward();
        let min = level.bounds.center - level.bounds.half_extents;
        let max = level.bounds.center + level.bounds.half_extents;

        assert!(level.apothecary_start.x >= min.x);
        assert!(level.apothecary_start.x <= max.x);
        assert!(level.apothecary_start.y >= min.y);
        assert!(level.apothecary_start.y <= max.y);
    }

    #[test]
    fn prototype_level_validates() {
        let level = LevelDefinition::prototype_quarantine_ward();

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn prototype_level_file_loads_and_validates() {
        let level =
            LevelDefinition::from_ron_file("../../assets/levels/prototype_quarantine_ward.ron")
                .expect("prototype level should load");

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn exit_requires_objective_in_prototype_level() {
        let level = LevelDefinition::prototype_quarantine_ward();

        assert_eq!(
            level.exits[0].required_objectives,
            vec!["analyze_contaminant_sample".to_string()]
        );
    }

    #[test]
    fn validation_rejects_empty_exit_target() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.exits[0].target.clear();

        assert_eq!(level.validate(), Err(LevelValidationError::InvalidExit));
    }

    #[test]
    fn validation_rejects_empty_exit_entry_id() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.exits[0].entry_id.clear();

        assert_eq!(level.validate(), Err(LevelValidationError::InvalidExit));
    }

    #[test]
    fn validation_rejects_duplicate_pickup_ids() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.pickups[1].id = level.pickups[0].id.clone();

        assert_eq!(level.validate(), Err(LevelValidationError::InvalidEntityId));
    }

    #[test]
    fn validation_rejects_duplicate_contaminant_ids() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.contaminants[1].id = level.contaminants[0].id.clone();

        assert_eq!(level.validate(), Err(LevelValidationError::InvalidEntityId));
    }

    #[test]
    fn validation_rejects_duplicate_entry_ids() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.entry_points.push(level.entry_points[0].clone());

        assert_eq!(level.validate(), Err(LevelValidationError::InvalidEntityId));
    }

    #[test]
    fn validation_rejects_invalid_spawn_interval() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.spawn_interval_seconds = Some(0.0);

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::InvalidSpawnInterval)
        );
    }
}
