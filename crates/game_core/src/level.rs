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
    #[serde(default)]
    pub decor: Vec<DecorDefinition>,
    #[serde(default)]
    pub sections: Vec<SectionDefinition>,
    pub entry_points: Vec<LevelEntryPoint>,
    pub exits: Vec<LevelExit>,
    #[serde(default)]
    pub transitions: Vec<LevelTransition>,
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
pub struct LevelTransition {
    pub id: String,
    pub position: Vec2,
    pub half_extents: Vec2,
    pub target: String,
    pub entry_id: String,
    #[serde(default)]
    pub kind: TransitionKind,
    #[serde(default)]
    pub required_objectives: Vec<String>,
    #[serde(default)]
    pub required_clearance: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransitionKind {
    #[default]
    Lift,
    Teleporter,
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
    #[serde(default)]
    pub kind: DoorKind,
    #[serde(default)]
    pub required_objectives: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DoorKind {
    #[default]
    Bulkhead,
    EnergyBarrier,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DecorDefinition {
    pub id: String,
    pub position: Vec2,
    pub kind: DecorKind,
    #[serde(default)]
    pub rotation_degrees: f32,
    #[serde(default)]
    pub blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SectionDefinition {
    pub id: String,
    pub label: String,
    pub bounds: AxisAlignedBox,
    pub kind: SectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum SectionKind {
    Corridor,
    Lab,
    Triage,
    Supply,
    Lift,
    Containment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DecorKind {
    BloodDrops,
    BloodSmear,
    BloodPool,
    AcidScorch,
    CrackedPanel,
    LabTable,
    MedBed,
    BioTank,
    SupplyCrate,
    PipeCluster,
    CorpsePile,
    FloorGrate,
    HazardFloor,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TerminalDefinition {
    pub id: String,
    pub position: Vec2,
    pub kind: TerminalKind,
    pub objective_id: Option<String>,
    #[serde(default)]
    pub actions: Vec<LevelEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TerminalKind {
    LabAnalyzer,
    ShipLog,
    SupplyConsole,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum LevelEvent {
    CompleteObjective(String),
    GrantClearance(String),
    UnlockDoor(String),
    AddAmmo(i32),
    Heal(i32),
    AcquireAreaScan,
    SetSpawnInterval(f32),
}

pub type TerminalAction = LevelEvent;

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

        let available_clearances: HashSet<String> = self
            .pickups
            .iter()
            .filter_map(|pickup| match &pickup.kind {
                PickupKind::SecurityKeycard(clearance_id) => Some(clearance_id.clone()),
                _ => None,
            })
            .collect();
        let mut door_ids = HashSet::new();
        for door in &self.doors {
            if door.id.trim().is_empty() || !door_ids.insert(door.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            if door.clearance_id.trim().is_empty() {
                return Err(LevelValidationError::InvalidClearanceId);
            }

            if door
                .required_objectives
                .iter()
                .any(|objective_id| objective_id.trim().is_empty())
            {
                return Err(LevelValidationError::InvalidObjective);
            }

            if door.starts_locked
                && door.clearance_id != "open"
                && !available_clearances.contains(&door.clearance_id)
            {
                return Err(LevelValidationError::UnreachableClearance);
            }

            if !door_has_matching_wall(door, &self.walls) {
                return Err(LevelValidationError::InvalidDoorPlacement);
            }

            if !door_has_clear_approaches(door, &self.walls) {
                return Err(LevelValidationError::BlockedDoorApproach);
            }
        }

        let mut terminal_ids = HashSet::new();
        let mut objective_completers = HashSet::new();
        for terminal in &self.terminals {
            if terminal.id.trim().is_empty() || !terminal_ids.insert(terminal.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            if matches!(terminal.objective_id, Some(ref objective_id) if objective_id.trim().is_empty())
            {
                return Err(LevelValidationError::InvalidObjective);
            }

            if let Some(objective_id) = &terminal.objective_id {
                objective_completers.insert(objective_id.clone());
            }

            for action in &terminal.actions {
                match action {
                    LevelEvent::CompleteObjective(objective_id) => {
                        if objective_id.trim().is_empty() {
                            return Err(LevelValidationError::InvalidObjective);
                        }
                        objective_completers.insert(objective_id.clone());
                    }
                    LevelEvent::GrantClearance(clearance_id) => {
                        if clearance_id.trim().is_empty() {
                            return Err(LevelValidationError::InvalidClearanceId);
                        }
                    }
                    LevelEvent::UnlockDoor(door_id) => {
                        if door_id.trim().is_empty() {
                            return Err(LevelValidationError::InvalidDoorReference);
                        }
                    }
                    LevelEvent::AddAmmo(amount) | LevelEvent::Heal(amount) if *amount <= 0 => {
                        return Err(LevelValidationError::InvalidTerminalAction);
                    }
                    LevelEvent::SetSpawnInterval(seconds) if *seconds <= 0.0 => {
                        return Err(LevelValidationError::InvalidTerminalAction);
                    }
                    _ => {}
                }
            }
        }

        let mut objective_ids = HashSet::new();
        for objective in &self.objectives {
            if objective.id.trim().is_empty()
                || objective.label.trim().is_empty()
                || !objective_ids.insert(objective.id.clone())
            {
                return Err(LevelValidationError::InvalidObjective);
            }
        }

        for objective_id in &objective_completers {
            if !objective_ids.contains(objective_id) {
                return Err(LevelValidationError::UnknownObjectiveReference);
            }
        }

        let mut decor_ids = HashSet::new();
        for decor in &self.decor {
            if decor.id.trim().is_empty() || !decor_ids.insert(decor.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            if !point_inside_box(decor.position, self.bounds) {
                return Err(LevelValidationError::DecorOutsideBounds);
            }

            if decor.blocking && decor_blocks_interaction_space(decor, self) {
                return Err(LevelValidationError::InvalidDecorPlacement);
            }
        }

        let mut section_ids = HashSet::new();
        for section in &self.sections {
            if section.id.trim().is_empty()
                || section.label.trim().is_empty()
                || !section_ids.insert(section.id.clone())
            {
                return Err(LevelValidationError::InvalidSection);
            }

            if !box_inside_box(section.bounds, self.bounds) {
                return Err(LevelValidationError::InvalidSection);
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

            if exit
                .required_objectives
                .iter()
                .any(|objective_id| !objective_ids.contains(objective_id))
            {
                return Err(LevelValidationError::UnknownObjectiveReference);
            }
        }

        let mut transition_ids = HashSet::new();
        for transition in &self.transitions {
            if transition.id.trim().is_empty() || !transition_ids.insert(transition.id.clone()) {
                return Err(LevelValidationError::InvalidEntityId);
            }

            if transition.target.trim().is_empty() || transition.entry_id.trim().is_empty() {
                return Err(LevelValidationError::InvalidTransition);
            }

            if matches!(transition.required_clearance, Some(ref clearance_id) if clearance_id.trim().is_empty())
            {
                return Err(LevelValidationError::InvalidClearanceId);
            }

            if let Some(clearance_id) = &transition.required_clearance {
                if clearance_id != "open" && !available_clearances.contains(clearance_id) {
                    return Err(LevelValidationError::UnreachableClearance);
                }
            }

            if transition
                .required_objectives
                .iter()
                .any(|objective_id| objective_id.trim().is_empty())
            {
                return Err(LevelValidationError::InvalidObjective);
            }

            if transition
                .required_objectives
                .iter()
                .any(|objective_id| !objective_ids.contains(objective_id))
            {
                return Err(LevelValidationError::UnknownObjectiveReference);
            }

            if !point_inside_box(transition.position, self.bounds)
                || self
                    .walls
                    .iter()
                    .any(|wall| point_inside_box(transition.position, *wall))
            {
                return Err(LevelValidationError::InvalidTransition);
            }
        }

        for door in &self.doors {
            if door
                .required_objectives
                .iter()
                .any(|objective_id| !objective_ids.contains(objective_id))
            {
                return Err(LevelValidationError::UnknownObjectiveReference);
            }
        }

        let door_ids: HashSet<&str> = self.doors.iter().map(|door| door.id.as_str()).collect();
        for terminal in &self.terminals {
            for action in &terminal.actions {
                if let LevelEvent::UnlockDoor(door_id) = action {
                    if !door_ids.contains(door_id.as_str()) {
                        return Err(LevelValidationError::InvalidDoorReference);
                    }
                }
            }
        }

        for objective in self
            .objectives
            .iter()
            .filter(|objective| objective.required)
        {
            if !objective_completers.contains(&objective.id) {
                return Err(LevelValidationError::UnreachableObjective);
            }
        }

        if matches!(self.spawn_interval_seconds, Some(seconds) if seconds <= 0.0) {
            return Err(LevelValidationError::InvalidSpawnInterval);
        }

        if !level_has_reachable_critical_points(self) {
            return Err(LevelValidationError::UnreachableInteraction);
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
                kind: DoorKind::Bulkhead,
                required_objectives: vec![],
            }],
            terminals: vec![TerminalDefinition {
                id: "ward_lab_analyzer".to_string(),
                position: Vec2::new(360.0, -96.0),
                kind: TerminalKind::LabAnalyzer,
                objective_id: Some("analyze_contaminant_sample".to_string()),
                actions: vec![],
            }],
            objectives: vec![ObjectiveDefinition {
                id: "analyze_contaminant_sample".to_string(),
                label: "Analyze contaminant sample".to_string(),
                required: true,
            }],
            decor: vec![],
            sections: vec![],
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
            transitions: vec![],
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
    DecorOutsideBounds,
    InvalidExit,
    InvalidSpawnInterval,
    UnknownObjectiveReference,
    UnreachableClearance,
    UnreachableObjective,
    InvalidDoorPlacement,
    BlockedDoorApproach,
    InvalidDecorPlacement,
    InvalidTerminalAction,
    InvalidTransition,
    UnreachableInteraction,
    InvalidDoorReference,
    InvalidSection,
}

const fn wall(x: f32, y: f32, width: f32, height: f32) -> AxisAlignedBox {
    AxisAlignedBox::new(Vec2::new(x, y), Vec2::new(width * 0.5, height * 0.5))
}

fn point_inside_box(point: Vec2, area: AxisAlignedBox) -> bool {
    let min = area.center - area.half_extents;
    let max = area.center + area.half_extents;

    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

fn box_inside_box(inner: AxisAlignedBox, outer: AxisAlignedBox) -> bool {
    point_inside_box(inner.center - inner.half_extents, outer)
        && point_inside_box(inner.center + inner.half_extents, outer)
}

fn door_has_matching_wall(door: &DoorDefinition, walls: &[AxisAlignedBox]) -> bool {
    let door_is_horizontal = door.half_extents.x >= door.half_extents.y;
    let door_bounds = AxisAlignedBox::new(door.position, door.half_extents);

    walls.iter().any(|wall| {
        let wall_is_horizontal = wall.half_extents.x >= wall.half_extents.y;
        wall_is_horizontal == door_is_horizontal && aabb_intersects(*wall, door_bounds)
    })
}

fn door_has_clear_approaches(door: &DoorDefinition, walls: &[AxisAlignedBox]) -> bool {
    door_approach_zones(door)
        .into_iter()
        .all(|zone| !walls.iter().any(|wall| aabb_intersects(*wall, zone)))
}

fn door_approach_zones(door: &DoorDefinition) -> [AxisAlignedBox; 2] {
    const APPROACH_DISTANCE: f32 = 30.0;
    const APPROACH_HALF_WIDTH: f32 = 18.0;
    const APPROACH_LONG_PADDING: f32 = 6.0;

    if door.half_extents.x >= door.half_extents.y {
        let half_extents = Vec2::new(
            door.half_extents.x + APPROACH_LONG_PADDING,
            APPROACH_HALF_WIDTH,
        );
        [
            AxisAlignedBox::new(
                door.position - Vec2::Y * (door.half_extents.y + APPROACH_DISTANCE),
                half_extents,
            ),
            AxisAlignedBox::new(
                door.position + Vec2::Y * (door.half_extents.y + APPROACH_DISTANCE),
                half_extents,
            ),
        ]
    } else {
        let half_extents = Vec2::new(
            APPROACH_HALF_WIDTH,
            door.half_extents.y + APPROACH_LONG_PADDING,
        );
        [
            AxisAlignedBox::new(
                door.position - Vec2::X * (door.half_extents.x + APPROACH_DISTANCE),
                half_extents,
            ),
            AxisAlignedBox::new(
                door.position + Vec2::X * (door.half_extents.x + APPROACH_DISTANCE),
                half_extents,
            ),
        ]
    }
}

fn aabb_intersects(a: AxisAlignedBox, b: AxisAlignedBox) -> bool {
    let a_min = a.center - a.half_extents;
    let a_max = a.center + a.half_extents;
    let b_min = b.center - b.half_extents;
    let b_max = b.center + b.half_extents;

    a_min.x <= b_max.x && a_max.x >= b_min.x && a_min.y <= b_max.y && a_max.y >= b_min.y
}

fn decor_blocks_interaction_space(decor: &DecorDefinition, level: &LevelDefinition) -> bool {
    const DECOR_BLOCKING_RADIUS: f32 = 46.0;
    const DOOR_CLEARANCE_RADIUS: f32 = 82.0;
    const EXIT_CLEARANCE_RADIUS: f32 = 72.0;
    const ENTRY_CLEARANCE_RADIUS: f32 = 72.0;
    const TERMINAL_CLEARANCE_RADIUS: f32 = 76.0;
    const PICKUP_CLEARANCE_RADIUS: f32 = 48.0;

    level.doors.iter().any(|door| {
        decor.position.distance(door.position) <= DECOR_BLOCKING_RADIUS + DOOR_CLEARANCE_RADIUS
    }) || level.exits.iter().any(|exit| {
        decor.position.distance(exit.position) <= DECOR_BLOCKING_RADIUS + EXIT_CLEARANCE_RADIUS
    }) || level.transitions.iter().any(|transition| {
        decor.position.distance(transition.position)
            <= DECOR_BLOCKING_RADIUS + EXIT_CLEARANCE_RADIUS
    }) || level.entry_points.iter().any(|entry| {
        decor.position.distance(entry.position) <= DECOR_BLOCKING_RADIUS + ENTRY_CLEARANCE_RADIUS
    }) || level.terminals.iter().any(|terminal| {
        decor.position.distance(terminal.position)
            <= DECOR_BLOCKING_RADIUS + TERMINAL_CLEARANCE_RADIUS
    }) || level.pickups.iter().any(|pickup| {
        decor.position.distance(pickup.position) <= DECOR_BLOCKING_RADIUS + PICKUP_CLEARANCE_RADIUS
    })
}

fn level_has_reachable_critical_points(level: &LevelDefinition) -> bool {
    let path_map = PathMap::new(level);
    let Some(reachable) = path_map.reachable_from(level.apothecary_start) else {
        return false;
    };

    level
        .entry_points
        .iter()
        .all(|entry| path_map.position_reachable(entry.position, 48.0, &reachable))
        && level
            .pickups
            .iter()
            .all(|pickup| path_map.position_reachable(pickup.position, 48.0, &reachable))
        && level
            .terminals
            .iter()
            .all(|terminal| path_map.position_reachable(terminal.position, 76.0, &reachable))
        && level.exits.iter().all(|exit| {
            path_map.position_reachable(
                exit.position,
                exit.half_extents.length() + 36.0,
                &reachable,
            )
        })
        && level.transitions.iter().all(|transition| {
            path_map.position_reachable(
                transition.position,
                transition.half_extents.length() + 36.0,
                &reachable,
            )
        })
}

struct PathMap<'a> {
    level: &'a LevelDefinition,
    origin: Vec2,
    width: usize,
    height: usize,
    cell_size: f32,
    passable: Vec<bool>,
}

impl<'a> PathMap<'a> {
    fn new(level: &'a LevelDefinition) -> Self {
        const CELL_SIZE: f32 = 24.0;

        let min = level.bounds.center - level.bounds.half_extents;
        let size = level.bounds.half_extents * 2.0;
        let width = (size.x / CELL_SIZE).ceil().max(1.0) as usize;
        let height = (size.y / CELL_SIZE).ceil().max(1.0) as usize;
        let mut path_map = Self {
            level,
            origin: min,
            width,
            height,
            cell_size: CELL_SIZE,
            passable: vec![false; width * height],
        };

        for y in 0..height {
            for x in 0..width {
                let position = path_map.cell_center(x, y);
                let is_passable =
                    point_inside_box(position, level.bounds) && !path_map.point_blocked(position);
                let index = path_map.index(x, y);
                path_map.passable[index] = is_passable;
            }
        }

        path_map
    }

    fn reachable_from(&self, start: Vec2) -> Option<Vec<bool>> {
        let start = self.nearest_passable_cell(start, 64.0)?;
        let mut reachable = vec![false; self.passable.len()];
        let mut frontier = vec![start];
        reachable[self.index(start.0, start.1)] = true;

        while let Some((x, y)) = frontier.pop() {
            for (next_x, next_y) in self.neighbor_cells(x, y) {
                let index = self.index(next_x, next_y);
                if !self.passable[index] || reachable[index] {
                    continue;
                }

                reachable[index] = true;
                frontier.push((next_x, next_y));
            }
        }

        Some(reachable)
    }

    fn position_reachable(&self, position: Vec2, radius: f32, reachable: &[bool]) -> bool {
        self.cells_within_radius(position, radius)
            .into_iter()
            .any(|(x, y)| reachable[self.index(x, y)])
    }

    fn nearest_passable_cell(&self, position: Vec2, radius: f32) -> Option<(usize, usize)> {
        self.cells_within_radius(position, radius)
            .into_iter()
            .find(|(x, y)| self.passable[self.index(*x, *y)])
    }

    fn cells_within_radius(&self, position: Vec2, radius: f32) -> Vec<(usize, usize)> {
        let min = position - Vec2::splat(radius.max(self.cell_size));
        let max = position + Vec2::splat(radius.max(self.cell_size));
        let min_cell = self.cell_for_position(min);
        let max_cell = self.cell_for_position(max);
        let mut cells = Vec::new();

        for y in min_cell.1..=max_cell.1 {
            for x in min_cell.0..=max_cell.0 {
                if self.cell_center(x, y).distance(position) <= radius + self.cell_size {
                    cells.push((x, y));
                }
            }
        }

        cells
    }

    fn neighbor_cells(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut cells = Vec::with_capacity(8);
        for y_offset in -1isize..=1 {
            for x_offset in -1isize..=1 {
                if x_offset == 0 && y_offset == 0 {
                    continue;
                }

                let next_x = x as isize + x_offset;
                let next_y = y as isize + y_offset;
                if next_x < 0
                    || next_y < 0
                    || next_x >= self.width as isize
                    || next_y >= self.height as isize
                {
                    continue;
                }
                cells.push((next_x as usize, next_y as usize));
            }
        }
        cells
    }

    fn point_blocked(&self, point: Vec2) -> bool {
        self.level
            .walls
            .iter()
            .any(|wall| point_inside_box(point, *wall) && !self.point_inside_door_opening(point))
            || self.level.decor.iter().any(|decor| {
                decor.blocking
                    && point.distance(decor.position) <= blocking_decor_radius(decor.kind)
            })
    }

    fn point_inside_door_opening(&self, point: Vec2) -> bool {
        self.level.doors.iter().any(|door| {
            let opening = AxisAlignedBox::new(door.position, door.half_extents + Vec2::splat(8.0));
            point_inside_box(point, opening)
        })
    }

    fn cell_for_position(&self, position: Vec2) -> (usize, usize) {
        let offset = position - self.origin;
        let x = (offset.x / self.cell_size)
            .floor()
            .clamp(0.0, (self.width - 1) as f32) as usize;
        let y = (offset.y / self.cell_size)
            .floor()
            .clamp(0.0, (self.height - 1) as f32) as usize;
        (x, y)
    }

    fn cell_center(&self, x: usize, y: usize) -> Vec2 {
        self.origin
            + Vec2::new(
                (x as f32 + 0.5) * self.cell_size,
                (y as f32 + 0.5) * self.cell_size,
            )
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

fn blocking_decor_radius(kind: DecorKind) -> f32 {
    match kind {
        DecorKind::LabTable | DecorKind::MedBed | DecorKind::BioTank => 48.0,
        DecorKind::SupplyCrate | DecorKind::PipeCluster | DecorKind::CorpsePile => 34.0,
        DecorKind::HazardFloor | DecorKind::FloorGrate => 24.0,
        DecorKind::BloodDrops
        | DecorKind::BloodSmear
        | DecorKind::BloodPool
        | DecorKind::AcidScorch
        | DecorKind::CrackedPanel => 18.0,
    }
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
    fn validation_rejects_locked_door_without_matching_keycard() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.pickups.retain(|pickup| {
            !matches!(pickup.kind, PickupKind::SecurityKeycard(ref id) if id == "quarantine_green")
        });

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::UnreachableClearance)
        );
    }

    #[test]
    fn validation_rejects_unknown_exit_objective() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.exits[0]
            .required_objectives
            .push("missing_objective".to_string());

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::UnknownObjectiveReference)
        );
    }

    #[test]
    fn validation_rejects_required_objective_without_completer() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.terminals[0].objective_id = None;

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::UnreachableObjective)
        );
    }

    #[test]
    fn validation_rejects_door_with_wrong_wall_orientation() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.doors[0].half_extents = Vec2::new(13.0, 32.0);

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::InvalidDoorPlacement)
        );
    }

    #[test]
    fn validation_rejects_door_with_blocked_approach() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.walls.push(wall(0.0, 125.0, 80.0, 24.0));

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::BlockedDoorApproach)
        );
    }

    #[test]
    fn validation_rejects_blocking_decor_near_terminal() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.decor.push(DecorDefinition {
            id: "bad_terminal_blocker".to_string(),
            position: level.terminals[0].position + Vec2::new(24.0, 0.0),
            kind: DecorKind::SupplyCrate,
            rotation_degrees: 0.0,
            blocking: true,
        });

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::InvalidDecorPlacement)
        );
    }

    #[test]
    fn validation_accepts_terminal_complete_objective_action() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.terminals[0].objective_id = None;
        level.terminals[0].actions = vec![TerminalAction::CompleteObjective(
            "analyze_contaminant_sample".to_string(),
        )];

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_invalid_terminal_action_amount() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.terminals[0].actions = vec![TerminalAction::AddAmmo(0)];

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::InvalidTerminalAction)
        );
    }

    #[test]
    fn validation_accepts_level_event_terminal_actions() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.terminals[0].actions = vec![
            LevelEvent::CompleteObjective("analyze_contaminant_sample".to_string()),
            LevelEvent::GrantClearance("quarantine_green".to_string()),
            LevelEvent::UnlockDoor("ward_quarantine_green_door".to_string()),
        ];

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_unknown_door_event_reference() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.terminals[0].actions = vec![
            LevelEvent::CompleteObjective("analyze_contaminant_sample".to_string()),
            LevelEvent::UnlockDoor("missing_door".to_string()),
        ];

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::InvalidDoorReference)
        );
    }

    #[test]
    fn validation_accepts_semantic_sections() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.sections.push(SectionDefinition {
            id: "ward_lab".to_string(),
            label: "Ward Lab".to_string(),
            bounds: AxisAlignedBox::new(Vec2::new(120.0, 0.0), Vec2::new(180.0, 120.0)),
            kind: SectionKind::Lab,
        });

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_section_outside_bounds() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.sections.push(SectionDefinition {
            id: "bad_section".to_string(),
            label: "Bad Section".to_string(),
            bounds: AxisAlignedBox::new(Vec2::new(700.0, 0.0), Vec2::new(80.0, 80.0)),
            kind: SectionKind::Corridor,
        });

        assert_eq!(level.validate(), Err(LevelValidationError::InvalidSection));
    }

    #[test]
    fn validation_accepts_reachable_transition() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.transitions.push(LevelTransition {
            id: "ward_lift".to_string(),
            position: Vec2::new(-310.0, -205.0),
            half_extents: Vec2::new(32.0, 32.0),
            target: "lab_access_corridor".to_string(),
            entry_id: "from_quarantine_ward".to_string(),
            kind: TransitionKind::Lift,
            required_objectives: vec!["analyze_contaminant_sample".to_string()],
            required_clearance: Some("quarantine_green".to_string()),
        });

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_unreachable_critical_points() {
        let mut level = LevelDefinition::prototype_quarantine_ward();
        level.walls.push(wall(150.0, 0.0, 24.0, 520.0));

        assert_eq!(
            level.validate(),
            Err(LevelValidationError::UnreachableInteraction)
        );
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
