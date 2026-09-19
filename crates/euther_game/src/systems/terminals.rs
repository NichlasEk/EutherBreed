use bevy::prelude::*;
use game_core::{LevelEvent, TerminalKind, TerminalPattern};

use crate::components::{Apothecary, Terminal};
use crate::resources::{
    ApothecaryVitals, ContaminantSpawnTimer, GameNotice, LevelRuntime, LocalLevelState,
};

const TERMINAL_INTERACTION_RADIUS: f32 = 42.0;

pub fn interact_with_terminals(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    assets: Res<AssetServer>,
    audio: Res<crate::audio_settings::AudioSettings>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    terminal_query: Query<(Entity, &Transform, &Terminal)>,
    mut level_state: ResMut<LocalLevelState>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut notice: ResMut<GameNotice>,
) {
    if !input.just_pressed(KeyCode::KeyE) {
        return;
    }

    let apothecary_position = apothecary_query.translation.xy();

    let Some((entity, transform, terminal)) = terminal_query
        .iter()
        .filter(|(_, t, _)| {
            apothecary_position.distance(t.translation.xy()) <= TERMINAL_INTERACTION_RADIUS
        })
        .min_by(|a, b| {
            apothecary_position
                .distance_squared(a.1.translation.xy())
                .total_cmp(&apothecary_position.distance_squared(b.1.translation.xy()))
        })
    else {
        return;
    };
    let response = response_for(terminal, &level_state, &vitals);
    let cue = match response {
        crate::terminal_visuals::Response::Success => "audio/terminal-analysis.ogg",
        crate::terminal_visuals::Response::Used => "audio/terminal-used.ogg",
        crate::terminal_visuals::Response::Denied => "audio/terminal-denied.ogg",
    };
    crate::audio_settings::play_sfx(&mut commands, &assets, &audio, cue);
    crate::terminal_visuals::respond(&mut commands, entity, response);
    if response != crate::terminal_visuals::Response::Success {
        return;
    }
    level_state.0.activate_terminal(terminal.id.clone());
    let actions = terminal_actions(terminal);
    let summary = execute_terminal_actions(
        &actions,
        &mut level_state,
        &mut level_runtime,
        &mut contaminant_timer,
        &mut vitals,
    );
    notice.show(
        match terminal.kind {
            TerminalKind::LabAnalyzer => "Analysis complete",
            TerminalKind::ShipLog => "Link established",
            TerminalKind::SupplyConsole => "Supplies released",
        },
        1.4,
    );
    info!(
        "terminal {} at {:?}: {}",
        terminal.id,
        transform.translation.xy(),
        summary
    );
}

fn response_for(
    terminal: &Terminal,
    state: &LocalLevelState,
    vitals: &ApothecaryVitals,
) -> crate::terminal_visuals::Response {
    use crate::terminal_visuals::Response;
    if state.0.activated_terminals.contains(&terminal.id) {
        Response::Used
    } else if vitals.0.bio_samples < terminal.required_bio_samples {
        Response::Denied
    } else {
        Response::Success
    }
}

pub(crate) fn terminal_actions_for_definition(
    terminal: &game_core::TerminalDefinition,
) -> Vec<LevelEvent> {
    terminal_actions(&Terminal {
        id: terminal.id.clone(),
        kind: terminal.kind.clone(),
        objective_id: terminal.objective_id.clone(),
        required_bio_samples: terminal.required_bio_samples,
        pattern: terminal.pattern.clone(),
        actions: terminal.actions.clone(),
    })
}

fn terminal_actions(terminal: &Terminal) -> Vec<LevelEvent> {
    if !terminal.actions.is_empty() {
        return terminal.actions.clone();
    }

    let mut actions = Vec::new();
    match terminal.pattern {
        TerminalPattern::Default => {
            if let Some(objective_id) = &terminal.objective_id {
                actions.push(LevelEvent::CompleteObjective(objective_id.clone()));
                actions.push(LevelEvent::SetSpawnInterval(2.2));
            }

            if matches!(terminal.kind, TerminalKind::SupplyConsole) {
                actions.push(LevelEvent::AddAmmo(16));
                actions.push(LevelEvent::Heal(18));
            }

            if actions.is_empty() && matches!(terminal.kind, TerminalKind::ShipLog) {
                actions.push(LevelEvent::AcquireAreaScan);
            }
        }
        TerminalPattern::SupplyStation => {
            actions.push(LevelEvent::AddAmmo(16));
            actions.push(LevelEvent::Heal(18));
        }
        TerminalPattern::ObjectiveRouter => {
            if let Some(objective_id) = &terminal.objective_id {
                actions.push(LevelEvent::CompleteObjective(objective_id.clone()));
                actions.push(LevelEvent::SetSpawnInterval(2.2));
            }
        }
        TerminalPattern::AreaScan => actions.push(LevelEvent::AcquireAreaScan),
    }

    actions
}

fn execute_terminal_actions(
    actions: &[LevelEvent],
    level_state: &mut LocalLevelState,
    level_runtime: &mut LevelRuntime,
    contaminant_timer: &mut ContaminantSpawnTimer,
    vitals: &mut ApothecaryVitals,
) -> String {
    let mut messages = Vec::new();

    for action in actions {
        match action {
            LevelEvent::CompleteObjective(objective_id) => {
                if level_state.0.complete_objective(objective_id.clone()) {
                    messages.push(format!("objective {objective_id} complete"));
                }
            }
            LevelEvent::GrantClearance(clearance_id) => {
                if level_state.0.grant_clearance(clearance_id.clone()) {
                    messages.push(format!("clearance {clearance_id} granted"));
                }
            }
            LevelEvent::UnlockDoor(door_id) => {
                if level_state.0.unlock_door(door_id.clone()) {
                    messages.push(format!("door {door_id} unlocked"));
                }
            }
            LevelEvent::AddAmmo(amount) => {
                vitals.0.add_ammo(*amount);
                messages.push(format!("ammo +{amount}"));
            }
            LevelEvent::Heal(amount) => {
                vitals.0.heal(*amount, 100);
                messages.push(format!("med-gel +{amount}"));
            }
            LevelEvent::AcquireAreaScan => {
                level_state.0.acquire_area_scan();
                messages.push("area scan uploaded".to_string());
            }
            LevelEvent::SetSpawnInterval(seconds) => {
                if level_runtime.dynamic_spawn_interval_seconds > 0.0 {
                    level_runtime.dynamic_spawn_interval_seconds =
                        level_runtime.dynamic_spawn_interval_seconds.min(*seconds);
                    contaminant_timer
                        .0
                        .set_duration(std::time::Duration::from_secs_f32(
                            level_runtime.dynamic_spawn_interval_seconds,
                        ));
                    contaminant_timer.0.reset();
                    messages.push("contamination surge".to_string());
                }
            }
        }
    }

    if messages.is_empty() {
        "Terminal processed".to_string()
    } else {
        messages.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_visuals::Response;
    #[test]
    fn used_terminal_reports_empty_even_if_samples_are_no_longer_available() {
        let terminal = Terminal {
            id: "analyzer".into(),
            kind: TerminalKind::LabAnalyzer,
            objective_id: None,
            required_bio_samples: 1,
            pattern: TerminalPattern::Default,
            actions: vec![],
        };
        let mut state = LocalLevelState::default();
        let mut vitals = ApothecaryVitals(game_core::ApothecaryVitals::new(100, 48, 0));
        assert_eq!(response_for(&terminal, &state, &vitals), Response::Denied);
        vitals.0.bio_samples = 1;
        assert_eq!(response_for(&terminal, &state, &vitals), Response::Success);
        state.0.activate_terminal("analyzer");
        vitals.0.bio_samples = 0;
        assert_eq!(response_for(&terminal, &state, &vitals), Response::Used);
    }
}
