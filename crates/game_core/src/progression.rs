use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ObjectiveProgress {
    completed: HashSet<String>,
}

impl ObjectiveProgress {
    pub fn complete(&mut self, objective_id: impl Into<String>) -> bool {
        let objective_id = objective_id.into();

        if objective_id.trim().is_empty() {
            return false;
        }

        self.completed.insert(objective_id)
    }

    pub fn is_complete(&self, objective_id: &str) -> bool {
        self.completed.contains(objective_id)
    }

    pub fn exit_readiness<'a>(
        &self,
        required_objectives: impl IntoIterator<Item = &'a String>,
    ) -> ExitReadiness {
        let missing: Vec<String> = required_objectives
            .into_iter()
            .filter(|objective_id| !self.is_complete(objective_id))
            .cloned()
            .collect();

        if missing.is_empty() {
            ExitReadiness::Ready
        } else {
            ExitReadiness::Blocked { missing }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitReadiness {
    Ready,
    Blocked { missing: Vec<String> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completing_objective_is_idempotent() {
        let mut progress = ObjectiveProgress::default();

        assert!(progress.complete("analyze_contaminant_sample"));
        assert!(!progress.complete("analyze_contaminant_sample"));
        assert!(progress.is_complete("analyze_contaminant_sample"));
    }

    #[test]
    fn empty_objective_id_is_ignored() {
        let mut progress = ObjectiveProgress::default();

        assert!(!progress.complete(""));
    }

    #[test]
    fn exit_ready_when_all_required_objectives_complete() {
        let mut progress = ObjectiveProgress::default();
        let required = vec!["a".to_string(), "b".to_string()];

        progress.complete("a");
        progress.complete("b");

        assert_eq!(progress.exit_readiness(&required), ExitReadiness::Ready);
    }

    #[test]
    fn exit_blocked_lists_missing_objectives() {
        let mut progress = ObjectiveProgress::default();
        let required = vec!["a".to_string(), "b".to_string()];

        progress.complete("a");

        assert_eq!(
            progress.exit_readiness(&required),
            ExitReadiness::Blocked {
                missing: vec!["b".to_string()]
            }
        );
    }
}
