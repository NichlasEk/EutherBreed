use serde::{Deserialize, Serialize};

use crate::{ApothecaryVitals, LevelState};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RuleGate {
    #[serde(default)]
    pub requirements: Vec<RuleRequirement>,
}

impl RuleGate {
    pub fn open() -> Self {
        Self::default()
    }

    pub fn with_clearance(clearance_id: &str) -> Self {
        if clearance_id == "open" {
            return Self::open();
        }

        Self {
            requirements: vec![RuleRequirement::Clearance(clearance_id.to_string())],
        }
    }

    pub fn for_door(clearance_id: &str, required_objectives: &[String]) -> Self {
        Self::for_clearance_and_objectives(clearance_id, required_objectives)
    }

    pub fn for_clearance_and_objectives(
        clearance_id: &str,
        required_objectives: &[String],
    ) -> Self {
        let mut gate = Self::with_clearance(clearance_id);
        gate.requirements.extend(
            required_objectives
                .iter()
                .cloned()
                .map(RuleRequirement::Objective),
        );
        gate
    }

    pub fn for_objectives(required_objectives: &[String]) -> Self {
        Self {
            requirements: required_objectives
                .iter()
                .cloned()
                .map(RuleRequirement::Objective)
                .collect(),
        }
    }

    pub fn evaluate(&self, context: RuleContext<'_>) -> RuleGateStatus {
        let missing: Vec<RuleRequirement> = self
            .requirements
            .iter()
            .filter(|requirement| !requirement.is_met(context))
            .cloned()
            .collect();

        if missing.is_empty() {
            RuleGateStatus::Open
        } else {
            RuleGateStatus::Blocked { missing }
        }
    }

    pub fn is_open(&self, context: RuleContext<'_>) -> bool {
        matches!(self.evaluate(context), RuleGateStatus::Open)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum RuleRequirement {
    Clearance(String),
    Objective(String),
    BioSamplesAtLeast(i32),
    AreaScan,
}

impl RuleRequirement {
    pub fn is_met(&self, context: RuleContext<'_>) -> bool {
        match self {
            RuleRequirement::Clearance(clearance_id) => {
                clearance_id == "open" || context.level_state.has_clearance(clearance_id)
            }
            RuleRequirement::Objective(objective_id) => {
                context.level_state.objectives.is_complete(objective_id)
            }
            RuleRequirement::BioSamplesAtLeast(required) => {
                context.vitals.bio_samples >= (*required).max(0)
            }
            RuleRequirement::AreaScan => context.level_state.area_scan_acquired,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RuleContext<'a> {
    pub level_state: &'a LevelState,
    pub vitals: &'a ApothecaryVitals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleGateStatus {
    Open,
    Blocked { missing: Vec<RuleRequirement> },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context<'a>(level_state: &'a LevelState, vitals: &'a ApothecaryVitals) -> RuleContext<'a> {
        RuleContext {
            level_state,
            vitals,
        }
    }

    #[test]
    fn open_gate_has_no_requirements() {
        let state = LevelState::default();
        let vitals = ApothecaryVitals::new(100, 48, 0);

        assert!(RuleGate::open().is_open(context(&state, &vitals)));
    }

    #[test]
    fn door_gate_checks_clearance_and_objectives() {
        let mut state = LevelState::default();
        let vitals = ApothecaryVitals::new(100, 48, 0);
        let gate = RuleGate::for_door("lab_lift", &["route_power".to_string()]);

        assert_eq!(
            gate.evaluate(context(&state, &vitals)),
            RuleGateStatus::Blocked {
                missing: vec![
                    RuleRequirement::Clearance("lab_lift".to_string()),
                    RuleRequirement::Objective("route_power".to_string())
                ]
            }
        );

        state.grant_clearance("lab_lift");
        state.complete_objective("route_power");

        assert!(gate.is_open(context(&state, &vitals)));
    }

    #[test]
    fn bio_sample_requirement_uses_vitals() {
        let state = LevelState::default();
        let vitals = ApothecaryVitals::new(100, 48, 2);
        let gate = RuleGate {
            requirements: vec![RuleRequirement::BioSamplesAtLeast(2)],
        };

        assert!(gate.is_open(context(&state, &vitals)));
    }
}
