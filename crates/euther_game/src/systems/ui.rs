use bevy::prelude::*;
use game_core::ExitReadiness;

use crate::components::{
    Apothecary, Door, ExitZone, NoticeText, ObjectiveText, PromptText, SectionText, StatusText,
    Terminal,
};
use crate::resources::{
    ApothecaryVitals, CampaignRuntime, CurrentLevelMap, GameNotice, LevelRuntime, LocalLevelState,
};

const PROMPT_RADIUS: f32 = 72.0;

pub fn update_status_text(
    level_state: Res<LocalLevelState>,
    vitals: Res<ApothecaryVitals>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if !vitals.is_changed() && !level_state.is_changed() {
        return;
    }

    for mut text in &mut text_query {
        **text = format!(
            "1UP <{}>  AMMO <{}>  KEYS {:02}  BIO {:02}",
            meter(vitals.0.health, 100, 12),
            meter(vitals.0.ammo, 48, 12),
            level_state.0.clearances.len(),
            vitals.0.bio_samples
        );
    }
}

pub fn update_section_text(
    runtime: Res<CampaignRuntime>,
    level_runtime: Res<LevelRuntime>,
    mut text_query: Query<&mut Text, With<SectionText>>,
) {
    if !runtime.is_changed() && !level_runtime.is_changed() {
        return;
    }

    let exits = if level_runtime.available_exits.is_empty() {
        "none".to_string()
    } else {
        level_runtime.available_exits.join(", ")
    };

    for mut text in &mut text_query {
        **text = format!(
            "SECTION <{}>  EXITS <{}>",
            runtime.progress.current_level(),
            exits
        );
    }
}

pub fn update_notice_text(
    time: Res<Time>,
    mut notice: ResMut<GameNotice>,
    mut text_query: Query<&mut Text, With<NoticeText>>,
) {
    if notice.is_visible() {
        notice.timer.tick(time.delta());

        if notice.timer.is_finished() {
            notice.clear();
        }
    }

    if !notice.is_changed() {
        return;
    }

    for mut text in &mut text_query {
        **text = notice.text.clone();
    }
}

pub fn update_objective_text(
    current_map: Res<CurrentLevelMap>,
    level_state: Res<LocalLevelState>,
    mut text_query: Query<&mut Text, With<ObjectiveText>>,
) {
    if !current_map.is_changed() && !level_state.is_changed() {
        return;
    }

    let text = current_map
        .level
        .as_ref()
        .and_then(|level| {
            level
                .objectives
                .iter()
                .find(|objective| {
                    objective.required && !level_state.0.objectives.is_complete(&objective.id)
                })
                .map(|objective| format!("OBJ <{}>", objective.label))
        })
        .unwrap_or_else(|| "OBJ <Reach the exit>".to_string());

    for mut current_text in &mut text_query {
        **current_text = text.clone();
    }
}

pub fn update_prompt_text(
    apothecary_query: Query<&Transform, With<Apothecary>>,
    terminal_query: Query<(&Transform, &Terminal)>,
    door_query: Query<(&Transform, &Door)>,
    exit_query: Query<(&Transform, &ExitZone)>,
    level_state: Res<LocalLevelState>,
    mut text_query: Query<&mut Text, With<PromptText>>,
) {
    let prompt = apothecary_query
        .single()
        .ok()
        .and_then(|transform| {
            prompt_for_position(
                transform.translation.xy(),
                &terminal_query,
                &door_query,
                &exit_query,
                &level_state,
            )
        })
        .unwrap_or_default();

    for mut text in &mut text_query {
        **text = prompt.clone();
    }
}

fn prompt_for_position(
    apothecary_position: Vec2,
    terminal_query: &Query<(&Transform, &Terminal)>,
    door_query: &Query<(&Transform, &Door)>,
    exit_query: &Query<(&Transform, &ExitZone)>,
    level_state: &LocalLevelState,
) -> Option<String> {
    for (transform, terminal) in terminal_query {
        if apothecary_position.distance(transform.translation.xy()) <= PROMPT_RADIUS {
            if level_state.0.activated_terminals.contains(&terminal.id) {
                return Some("TERMINAL <processed>".to_string());
            }
            return Some("PRESS E <analyze terminal>".to_string());
        }
    }

    for (transform, door) in door_query {
        if apothecary_position.distance(transform.translation.xy()) <= PROMPT_RADIUS {
            if door.locked {
                return Some("DOOR <locked - find clearance>".to_string());
            }
            return Some("DOOR <clearance accepted>".to_string());
        }
    }

    for (transform, exit) in exit_query {
        if apothecary_position.distance(transform.translation.xy()) <= PROMPT_RADIUS {
            return Some(
                match level_state
                    .0
                    .objectives
                    .exit_readiness(&exit.required_objectives)
                {
                    ExitReadiness::Ready => "EXIT <ready>".to_string(),
                    ExitReadiness::Blocked { .. } => {
                        "EXIT <locked - objective incomplete>".to_string()
                    }
                },
            );
        }
    }

    None
}

fn meter(value: i32, max: i32, width: usize) -> String {
    let filled = ((value.max(0) as f32 / max.max(1) as f32) * width as f32).ceil() as usize;
    let filled = filled.min(width);
    format!("{}{}", "/".repeat(filled), ".".repeat(width - filled))
}
