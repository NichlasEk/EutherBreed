use std::collections::HashSet;

use crate::{ApothecaryVitals, ObjectiveProgress};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RunState {
    pub vitals: ApothecaryVitals,
    pub current_level: String,
}

impl RunState {
    pub fn new(vitals: ApothecaryVitals, current_level: impl Into<String>) -> Self {
        Self {
            vitals,
            current_level: current_level.into(),
        }
    }

    pub fn travel_to(&mut self, level_id: impl Into<String>) {
        self.current_level = level_id.into();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LevelState {
    pub clearances: HashSet<String>,
    pub objectives: ObjectiveProgress,
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

    pub fn reset_for_level_travel(&mut self) {
        self.clearances.clear();
        self.objectives = ObjectiveProgress::default();
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

        state.reset_for_level_travel();

        assert!(!state.has_clearance("quarantine_green"));
        assert!(!state.objectives.is_complete("analyze_contaminant_sample"));
    }

    #[test]
    fn run_state_keeps_vitals_when_traveling() {
        let mut run = RunState::new(ApothecaryVitals::new(74, 12, 3), "a");

        run.travel_to("b");

        assert_eq!(run.current_level, "b");
        assert_eq!(run.vitals, ApothecaryVitals::new(74, 12, 3));
    }

    #[test]
    fn empty_clearance_is_ignored() {
        let mut state = LevelState::default();

        assert!(!state.grant_clearance(""));
    }
}
