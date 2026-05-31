use std::collections::HashSet;

use crate::{ApothecaryVitals, ObjectiveProgress};
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RunState {
    pub vitals: ApothecaryVitals,
    pub current_level: String,
    #[serde(default)]
    pub position: Vec2,
}

impl RunState {
    pub fn new(vitals: ApothecaryVitals, current_level: impl Into<String>) -> Self {
        Self::new_at(vitals, current_level, Vec2::ZERO)
    }

    pub fn new_at(
        vitals: ApothecaryVitals,
        current_level: impl Into<String>,
        position: Vec2,
    ) -> Self {
        Self {
            vitals,
            current_level: current_level.into(),
            position,
        }
    }

    pub fn travel_to(&mut self, level_id: impl Into<String>) {
        self.current_level = level_id.into();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LevelState {
    #[serde(default)]
    pub clearances: HashSet<String>,
    #[serde(default)]
    pub objectives: ObjectiveProgress,
    #[serde(default)]
    pub collected_pickups: HashSet<String>,
    #[serde(default)]
    pub unlocked_doors: HashSet<String>,
    #[serde(default)]
    pub activated_terminals: HashSet<String>,
    #[serde(default)]
    pub killed_contaminants: HashSet<String>,
    #[serde(default)]
    pub area_scan_acquired: bool,
}

impl LevelState {
    pub fn grant_clearance(&mut self, clearance_id: impl Into<String>) -> bool {
        let clearance_id = clearance_id.into();

        if clearance_id.trim().is_empty() {
            return false;
        }

        self.clearances.insert(clearance_id)
    }

    pub fn has_clearance(&self, clearance_id: &str) -> bool {
        self.clearances.contains(clearance_id)
    }

    pub fn complete_objective(&mut self, objective_id: impl Into<String>) -> bool {
        self.objectives.complete(objective_id)
    }

    pub fn collect_pickup(&mut self, pickup_id: impl Into<String>) -> bool {
        let pickup_id = pickup_id.into();

        if pickup_id.trim().is_empty() {
            return false;
        }

        self.collected_pickups.insert(pickup_id)
    }

    pub fn has_collected_pickup(&self, pickup_id: &str) -> bool {
        self.collected_pickups.contains(pickup_id)
    }

    pub fn unlock_door(&mut self, door_id: impl Into<String>) -> bool {
        let door_id = door_id.into();

        if door_id.trim().is_empty() {
            return false;
        }

        self.unlocked_doors.insert(door_id)
    }

    pub fn has_unlocked_door(&self, door_id: &str) -> bool {
        self.unlocked_doors.contains(door_id)
    }

    pub fn activate_terminal(&mut self, terminal_id: impl Into<String>) -> bool {
        let terminal_id = terminal_id.into();

        if terminal_id.trim().is_empty() {
            return false;
        }

        self.activated_terminals.insert(terminal_id)
    }

    pub fn has_activated_terminal(&self, terminal_id: &str) -> bool {
        self.activated_terminals.contains(terminal_id)
    }

    pub fn kill_contaminant(&mut self, contaminant_id: impl Into<String>) -> bool {
        let contaminant_id = contaminant_id.into();

        if contaminant_id.trim().is_empty() {
            return false;
        }

        self.killed_contaminants.insert(contaminant_id)
    }

    pub fn has_killed_contaminant(&self, contaminant_id: &str) -> bool {
        self.killed_contaminants.contains(contaminant_id)
    }

    pub fn acquire_area_scan(&mut self) {
        self.area_scan_acquired = true;
    }

    pub fn reset_for_level_travel(&mut self) {
        self.clearances.clear();
        self.objectives = ObjectiveProgress::default();
        self.collected_pickups.clear();
        self.unlocked_doors.clear();
        self.activated_terminals.clear();
        self.killed_contaminants.clear();
        self.area_scan_acquired = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_state_reset_clears_local_progress() {
        let mut state = LevelState::default();
        state.grant_clearance("quarantine_green");
        state.complete_objective("analyze_contaminant_sample");
        state.collect_pickup("ward_rounds_a");
        state.unlock_door("ward_quarantine_green_door");
        state.activate_terminal("ward_lab_analyzer");
        state.kill_contaminant("ward_contaminant_alpha");
        state.acquire_area_scan();

        state.reset_for_level_travel();

        assert!(!state.has_clearance("quarantine_green"));
        assert!(!state.objectives.is_complete("analyze_contaminant_sample"));
        assert!(!state.has_collected_pickup("ward_rounds_a"));
        assert!(!state.has_unlocked_door("ward_quarantine_green_door"));
        assert!(!state.has_activated_terminal("ward_lab_analyzer"));
        assert!(!state.has_killed_contaminant("ward_contaminant_alpha"));
        assert!(!state.area_scan_acquired);
    }

    #[test]
    fn run_state_keeps_vitals_when_traveling() {
        let mut run = RunState::new(ApothecaryVitals::new(74, 12, 3), "a");

        run.travel_to("b");

        assert_eq!(run.current_level, "b");
        assert_eq!(run.vitals, ApothecaryVitals::new(74, 12, 3));
    }

    #[test]
    fn run_state_tracks_position() {
        let run = RunState::new_at(ApothecaryVitals::new(74, 12, 3), "a", Vec2::new(4.0, -8.0));

        assert_eq!(run.position, Vec2::new(4.0, -8.0));
    }

    #[test]
    fn empty_clearance_is_ignored() {
        let mut state = LevelState::default();

        assert!(!state.grant_clearance(""));
    }

    #[test]
    fn collected_pickups_are_tracked_once() {
        let mut state = LevelState::default();

        assert!(state.collect_pickup("ward_rounds_a"));
        assert!(!state.collect_pickup("ward_rounds_a"));
        assert!(state.has_collected_pickup("ward_rounds_a"));
        assert!(!state.collect_pickup(""));
    }

    #[test]
    fn doors_and_terminals_are_tracked_once() {
        let mut state = LevelState::default();

        assert!(state.unlock_door("ward_quarantine_green_door"));
        assert!(!state.unlock_door("ward_quarantine_green_door"));
        assert!(state.has_unlocked_door("ward_quarantine_green_door"));
        assert!(!state.unlock_door(""));

        assert!(state.activate_terminal("ward_lab_analyzer"));
        assert!(!state.activate_terminal("ward_lab_analyzer"));
        assert!(state.has_activated_terminal("ward_lab_analyzer"));
        assert!(!state.activate_terminal(""));
    }

    #[test]
    fn killed_contaminants_are_tracked_once() {
        let mut state = LevelState::default();

        assert!(state.kill_contaminant("ward_contaminant_alpha"));
        assert!(!state.kill_contaminant("ward_contaminant_alpha"));
        assert!(state.has_killed_contaminant("ward_contaminant_alpha"));
        assert!(!state.kill_contaminant(""));
    }
}
