use bevy::prelude::*;
use game_core::{LevelEvent, TerminalKind};

use crate::components::{Apothecary, EffectLifetime, LevelEntity, Terminal};
use crate::resources::{
    ApothecaryVitals, ContaminantSpawnTimer, GameNotice, LevelRuntime, LocalLevelState,
};

const TERMINAL_INTERACTION_RADIUS: f32 = 42.0;

pub fn interact_with_terminals(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    terminal_query: Query<(&Transform, &Terminal)>,
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

    for (transform, terminal) in &terminal_query {
        if apothecary_position.distance(transform.translation.xy()) > TERMINAL_INTERACTION_RADIUS {
            continue;
        }

        if !level_state.0.activate_terminal(terminal.id.clone()) {
            notice.show("Terminal already processed", 1.4);
            continue;
        }

        spawn_terminal_activation_effect(&mut commands, transform.translation.xy(), &terminal.kind);
        let actions = terminal_actions(terminal);
        let summary = execute_terminal_actions(
            &actions,
            &mut level_state,
            &mut level_runtime,
            &mut contaminant_timer,
            &mut vitals,
        );
        notice.show(summary, 1.8);
        info!(
            "terminal {:?} executed actions {:?}",
            terminal.kind, actions
        );
    }
}

fn terminal_actions(terminal: &Terminal) -> Vec<LevelEvent> {
    if !terminal.actions.is_empty() {
        return terminal.actions.clone();
    }

    let mut actions = Vec::new();
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

fn spawn_terminal_activation_effect(commands: &mut Commands, position: Vec2, kind: &TerminalKind) {
    let color = match kind {
        TerminalKind::LabAnalyzer => Color::srgba(0.30, 1.0, 0.84, 0.62),
        TerminalKind::ShipLog => Color::srgba(0.45, 0.70, 1.0, 0.58),
        TerminalKind::SupplyConsole => Color::srgba(1.0, 0.72, 0.22, 0.62),
    };

    commands.spawn((
        Sprite::from_color(color, Vec2::new(70.0, 46.0)),
        Transform::from_xyz(position.x, position.y, 5.5),
        EffectLifetime(Timer::from_seconds(0.34, TimerMode::Once)),
        LevelEntity,
    ));
}
