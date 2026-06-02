use bevy::prelude::*;
use game_core::{AxisAlignedBox, RuleContext, RuleGate, RuleGateStatus, TransitionKind};

use crate::components::{Apothecary, LevelEntity, TransitionZone};
use crate::resources::{
    ApothecaryVitals, CampaignSignal, GameNotice, LocalLevelState, PendingExit, PendingTransition,
    TransitionTravel,
};

const APOTHECARY_RADIUS: f32 = 22.0;
const TRANSITION_SECONDS: f32 = 0.72;

pub fn trigger_transition_zones(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    transition_query: Query<(&Transform, &TransitionZone)>,
    level_state: Res<LocalLevelState>,
    vitals: Res<ApothecaryVitals>,
    mut pending_transition: ResMut<PendingTransition>,
    mut notice: ResMut<GameNotice>,
) {
    if !keys.just_pressed(KeyCode::KeyE) || pending_transition.travel.is_some() {
        return;
    }

    let apothecary_position = apothecary_query.translation.xy();
    for (transform, transition) in &transition_query {
        let bounds = AxisAlignedBox::new(transform.translation.xy(), transition.half_extents);
        if !point_inside_expanded_box(apothecary_position, bounds, APOTHECARY_RADIUS) {
            continue;
        }

        let clearance_id = transition.required_clearance.as_deref().unwrap_or("open");
        match RuleGate::for_door(clearance_id, &transition.required_objectives).evaluate(
            RuleContext {
                level_state: &level_state.0,
                vitals: &vitals.0,
            },
        ) {
            RuleGateStatus::Open => {
                pending_transition.travel = Some(TransitionTravel {
                    target: transition.target.clone(),
                    entry_id: transition.entry_id.clone(),
                    timer: Timer::from_seconds(TRANSITION_SECONDS, TimerMode::Once),
                });
                spawn_transition_effect(&mut commands, transform.translation.xy(), transition.kind);
                notice.show(
                    match transition.kind {
                        TransitionKind::Lift => "Lift engaged",
                        TransitionKind::Teleporter => "Transit lock acquired",
                    },
                    TRANSITION_SECONDS,
                );
                info!(
                    "transition id={} target={} entry={} engaged",
                    transition.id, transition.target, transition.entry_id
                );
                return;
            }
            RuleGateStatus::Blocked { .. } => {
                notice.show(
                    match transition.kind {
                        TransitionKind::Lift => "Lift locked: route clearance incomplete",
                        TransitionKind::Teleporter => "Transit locked: route clearance incomplete",
                    },
                    1.4,
                );
                return;
            }
        }
    }
}

pub fn update_pending_transition(
    time: Res<Time>,
    mut pending_transition: ResMut<PendingTransition>,
    mut campaign_signal: ResMut<CampaignSignal>,
) {
    let Some(travel) = pending_transition.travel.as_mut() else {
        return;
    };

    travel.timer.tick(time.delta());
    if !travel.timer.is_finished() {
        return;
    }

    campaign_signal.pending_exit = Some(PendingExit {
        target: travel.target.clone(),
        entry_id: travel.entry_id.clone(),
    });
    pending_transition.travel = None;
}

fn spawn_transition_effect(commands: &mut Commands, position: Vec2, kind: TransitionKind) {
    let (size, color) = match kind {
        TransitionKind::Lift => (Vec2::new(96.0, 96.0), Color::srgba(0.72, 0.96, 1.0, 0.42)),
        TransitionKind::Teleporter => (Vec2::new(82.0, 124.0), Color::srgba(0.18, 1.0, 0.92, 0.56)),
    };
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(position.x, position.y, 7.0),
        crate::components::EffectLifetime(Timer::from_seconds(TRANSITION_SECONDS, TimerMode::Once)),
        LevelEntity,
    ));
}

fn point_inside_expanded_box(point: Vec2, area: AxisAlignedBox, expansion: f32) -> bool {
    let min = area.center - area.half_extents - Vec2::splat(expansion);
    let max = area.center + area.half_extents + Vec2::splat(expansion);

    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}
