use bevy::prelude::*;
use game_core::{ExitReadiness, TerminalKind, TransitionKind};

use crate::components::{
    Apothecary, BioText, Door, DoorOpening, ExitZone, HudGaugeKind, HudGaugePip, KeysText,
    NoticeText, ObjectiveText, PromptText, SectionText, Terminal, TransitionZone,
};
use crate::resources::{
    ApothecaryVitals, CampaignRuntime, CurrentLevelMap, GameNotice, LevelRuntime, LocalLevelState,
};

const PROMPT_RADIUS: f32 = 72.0;

pub fn update_status_text(
    level_state: Res<LocalLevelState>,
    vitals: Res<ApothecaryVitals>,
    mut pip_query: Query<(&HudGaugePip, &mut BackgroundColor, &mut BorderColor)>,
    mut keys_query: Query<&mut Text, With<KeysText>>,
    mut bio_query: Query<&mut Text, (With<BioText>, Without<KeysText>)>,
) {
    if !vitals.is_changed() && !level_state.is_changed() {
        return;
    }

    for (pip, mut color, mut border) in &mut pip_query {
        let active = match pip.kind {
            HudGaugeKind::Health => active_pip(pip.index, vitals.0.health, 100, 12),
            HudGaugeKind::Ammo => active_pip(pip.index, vitals.0.ammo, 48, 12),
        };

        let (fill, edge) = match (pip.kind, active) {
            (HudGaugeKind::Health, true) => (
                Color::srgb(0.95, 0.39, 0.14),
                Color::srgba(1.0, 0.70, 0.24, 0.68),
            ),
            (HudGaugeKind::Ammo, true) => (
                Color::srgb(1.00, 0.55, 0.08),
                Color::srgba(1.0, 0.75, 0.24, 0.70),
            ),
            (HudGaugeKind::Health, false) => (
                Color::srgb(0.20, 0.08, 0.04),
                Color::srgba(0.52, 0.24, 0.12, 0.46),
            ),
            (HudGaugeKind::Ammo, false) => (
                Color::srgb(0.22, 0.10, 0.03),
                Color::srgba(0.58, 0.30, 0.10, 0.46),
            ),
        };

        color.0 = fill;
        *border = BorderColor::all(edge);
    }

    for mut text in &mut keys_query {
        **text = format!("{:02}", level_state.0.clearances.len());
    }

    for mut text in &mut bio_query {
        **text = format!("{:02}", vitals.0.bio_samples);
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
        **text = format!("{} | exits {}", runtime.progress.current_level(), exits);
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
                .map(|objective| objective.label.clone())
        })
        .unwrap_or_else(|| "Reach the exit".to_string());

    for mut current_text in &mut text_query {
        **current_text = text.clone();
    }
}

pub fn update_prompt_text(
    apothecary_query: Query<&Transform, With<Apothecary>>,
    terminal_query: Query<(&Transform, &Terminal)>,
    door_query: Query<(&Transform, &Door, Option<&DoorOpening>)>,
    exit_query: Query<(&Transform, &ExitZone)>,
    transition_query: Query<(&Transform, &TransitionZone)>,
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
                &transition_query,
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
    door_query: &Query<(&Transform, &Door, Option<&DoorOpening>)>,
    exit_query: &Query<(&Transform, &ExitZone)>,
    transition_query: &Query<(&Transform, &TransitionZone)>,
    level_state: &LocalLevelState,
) -> Option<String> {
    for (transform, terminal) in terminal_query {
        if apothecary_position.distance(transform.translation.xy()) <= PROMPT_RADIUS {
            if level_state.0.activated_terminals.contains(&terminal.id) {
                return Some("TERMINAL <processed>".to_string());
            }
            return Some(match terminal.kind {
                TerminalKind::LabAnalyzer => "PRESS E <analyze terminal>".to_string(),
                TerminalKind::ShipLog => "PRESS E <read ship log>".to_string(),
                TerminalKind::SupplyConsole => "PRESS E <use supply station>".to_string(),
            });
        }
    }

    for (transform, door, opening) in door_query {
        if apothecary_position.distance(transform.translation.xy()) <= PROMPT_RADIUS {
            if opening.is_some() {
                return Some("DOOR <opening>".to_string());
            }
            if door.opened {
                return Some("DOOR <open>".to_string());
            }
            if door.locked {
                return Some("DOOR <locked - find clearance>".to_string());
            }
            return Some("DOOR <clearance accepted - approach>".to_string());
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

    for (transform, transition) in transition_query {
        if apothecary_position.distance(transform.translation.xy()) <= PROMPT_RADIUS {
            let objective_ready = matches!(
                level_state
                    .0
                    .objectives
                    .exit_readiness(&transition.required_objectives),
                ExitReadiness::Ready
            );
            let clearance_ready =
                transition
                    .required_clearance
                    .as_deref()
                    .is_none_or(|clearance_id| {
                        clearance_id == "open" || level_state.0.has_clearance(clearance_id)
                    });
            if !objective_ready || !clearance_ready {
                return Some(match transition.kind {
                    TransitionKind::Lift => "LIFT <locked - route incomplete>".to_string(),
                    TransitionKind::Teleporter => "TRANSIT <locked - route incomplete>".to_string(),
                });
            }

            return Some(match transition.kind {
                TransitionKind::Lift => "PRESS E <use lift>".to_string(),
                TransitionKind::Teleporter => "PRESS E <use teleporter>".to_string(),
            });
        }
    }

    None
}

fn active_pip(index: usize, value: i32, max: i32, width: usize) -> bool {
    let filled = ((value.max(0) as f32 / max.max(1) as f32) * width as f32).ceil() as usize;
    index < filled.min(width)
}
