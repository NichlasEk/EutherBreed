use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::AxisAlignedBox;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LevelDefinition {
    pub name: String,
    pub bounds: AxisAlignedBox,
    pub apothecary_start: Vec2,
    pub walls: Vec<AxisAlignedBox>,
    pub contaminants: Vec<Vec2>,
    pub pickups: Vec<PrototypeEntity<PickupKind>>,
    pub doors: Vec<DoorDefinition>,
    pub exits: Vec<LevelExit>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PrototypeEntity<T> {
    pub position: Vec2,
    pub kind: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum PickupKind {
    ReagentRounds(i32),
    MedGel(i32),
    BioSample,
    SecurityKeycard(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LevelExit {
    pub position: Vec2,
    pub half_extents: Vec2,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DoorDefinition {
    pub position: Vec2,
    pub half_extents: Vec2,
    pub clearance_id: String,
    pub starts_locked: bool,
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

        for pickup in &self.pickups {
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

        for door in &self.doors {
            if door.clearance_id.trim().is_empty() {
                return Err(LevelValidationError::InvalidClearanceId);
            }
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
            contaminants: vec![Vec2::new(320.0, 180.0), Vec2::new(380.0, -185.0)],
            pickups: vec![
                PrototypeEntity {
                    position: Vec2::new(-70.0, -190.0),
                    kind: PickupKind::ReagentRounds(12),
                },
                PrototypeEntity {
                    position: Vec2::new(310.0, 80.0),
                    kind: PickupKind::MedGel(25),
                },
                PrototypeEntity {
                    position: Vec2::new(-310.0, 165.0),
                    kind: PickupKind::SecurityKeycard("quarantine_green".to_string()),
                },
            ],
            doors: vec![DoorDefinition {
                position: Vec2::new(0.0, 82.0),
                half_extents: Vec2::new(32.0, 13.0),
                clearance_id: "quarantine_green".to_string(),
                starts_locked: true,
            }],
            exits: vec![LevelExit {
                position: Vec2::new(432.0, 205.0),
                half_extents: Vec2::new(22.0, 46.0),
                target: "lab_access_corridor".to_string(),
            }],
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
    InvalidPickupAmount,
    InvalidClearanceId,
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
}
