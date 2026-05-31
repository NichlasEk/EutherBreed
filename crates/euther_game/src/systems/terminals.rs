use bevy::prelude::*;
use game_core::TerminalKind;

use crate::components::{Apothecary, Terminal};
use crate::resources::{
    ApothecaryVitals, ContaminantSpawnTimer, GameNotice, LevelRuntime, LocalLevelState,
};

const TERMINAL_INTERACTION_RADIUS: f32 = 42.0;

pub fn interact_with_terminals(
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

        if matches!(terminal.kind, TerminalKind::SupplyConsole) {
            vitals.0.add_ammo(16);
            vitals.0.heal(18, 100);
        }

        if let Some(objective_id) = &terminal.objective_id {
            if level_state.0.complete_objective(objective_id.clone()) {
                if level_runtime.dynamic_spawn_interval_seconds > 0.0 {
                    level_runtime.dynamic_spawn_interval_seconds =
                        level_runtime.dynamic_spawn_interval_seconds.min(2.2);
                    contaminant_timer
                        .0
                        .set_duration(std::time::Duration::from_secs_f32(
                            level_runtime.dynamic_spawn_interval_seconds,
                        ));
                    contaminant_timer.0.reset();
                }
                notice.show("Objective complete - contamination surge", 1.8);
                info!(
                    "terminal {:?} completed objective {}",
                    terminal.kind, objective_id
                );
            }
        } else {
            let message = match terminal.kind {
                TerminalKind::LabAnalyzer => "Analyzer accessed",
                TerminalKind::ShipLog => "Ship log recovered",
                TerminalKind::SupplyConsole => "Supply station: ammo +16 med-gel +18",
            };
            notice.show(message, 1.6);
        }
    }
}
