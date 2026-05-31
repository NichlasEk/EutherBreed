use glam::Vec2;

use crate::AxisAlignedBox;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelDefinition {
    pub name: &'static str,
    pub bounds: AxisAlignedBox,
    pub apothecary_start: Vec2,
    pub walls: Vec<AxisAlignedBox>,
    pub contaminants: Vec<Vec2>,
    pub pickups: Vec<PrototypeEntity<PickupKind>>,
    pub exits: Vec<LevelExit>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrototypeEntity<T> {
    pub position: Vec2,
    pub kind: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    ReagentRounds(i32),
    MedGel(i32),
    BioSample,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelExit {
    pub position: Vec2,
    pub half_extents: Vec2,
    pub target: &'static str,
}

impl LevelDefinition {
    pub fn prototype_quarantine_ward() -> Self {
        Self {
            name: "prototype_quarantine_ward",
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
            ],
            exits: vec![LevelExit {
                position: Vec2::new(432.0, 205.0),
                half_extents: Vec2::new(22.0, 46.0),
                target: "lab_access_corridor",
            }],
        }
    }
}

const fn wall(x: f32, y: f32, width: f32, height: f32) -> AxisAlignedBox {
    AxisAlignedBox::new(Vec2::new(x, y), Vec2::new(width * 0.5, height * 0.5))
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
}
