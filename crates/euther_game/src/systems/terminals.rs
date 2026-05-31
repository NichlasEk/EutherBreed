use bevy::prelude::*;

use crate::components::{Apothecary, Terminal};
use crate::resources::LocalLevelState;

const TERMINAL_INTERACTION_RADIUS: f32 = 42.0;

pub fn interact_with_terminals(
    input: Res<ButtonInput<KeyCode>>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    terminal_query: Query<(&Transform, &Terminal)>,
    mut level_state: ResMut<LocalLevelState>,
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
            continue;
        }

        if let Some(objective_id) = &terminal.objective_id {
            if level_state.0.complete_objective(objective_id.clone()) {
                info!(
                    "terminal {:?} completed objective {}",
                    terminal.kind, objective_id
                );
            }
        }
    }
}
