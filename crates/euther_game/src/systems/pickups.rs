use bevy::prelude::*;
use game_core::{ExitReadiness, PickupKind};

use crate::components::{Apothecary, Door, ExitZone, Pickup, Wall};
use crate::resources::{ApothecaryVitals, CampaignSignal, LocalLevelState};

const APOTHECARY_RADIUS: f32 = 22.0;
const PICKUP_RADIUS: f32 = 14.0;

pub fn collect_pickups(
    mut commands: Commands,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    pickup_query: Query<(Entity, &Transform, &Pickup)>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut level_state: ResMut<LocalLevelState>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (entity, transform, pickup) in &pickup_query {
        let distance = apothecary_position.distance(transform.translation.xy());

        if distance > APOTHECARY_RADIUS + PICKUP_RADIUS {
            continue;
        }

        match pickup.kind {
            PickupKind::ReagentRounds(amount) => vitals.0.add_ammo(amount),
            PickupKind::MedGel(amount) => vitals.0.heal(amount, 100),
            PickupKind::BioSample => vitals.0.collect_bio_sample(),
            PickupKind::SecurityKeycard(ref clearance_id) => {
                level_state.0.grant_clearance(clearance_id.clone());
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn unlock_doors(
    mut commands: Commands,
    level_state: Res<LocalLevelState>,
    mut door_query: Query<(Entity, &mut Door, &mut Sprite), With<Wall>>,
) {
    if !level_state.is_changed() {
        return;
    }

    for (entity, mut door, mut sprite) in &mut door_query {
        if !door.locked || !level_state.0.has_clearance(&door.clearance_id) {
            continue;
        }

        door.locked = false;
        sprite.color = Color::srgba(0.20, 0.58, 0.62, 0.25);
        commands.entity(entity).remove::<Wall>();
    }
}

pub fn report_exit_overlap(
    apothecary_query: Single<&Transform, With<Apothecary>>,
    exit_query: Query<(&Transform, &ExitZone)>,
    level_state: Res<LocalLevelState>,
    mut campaign_signal: ResMut<CampaignSignal>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (transform, exit) in &exit_query {
        if apothecary_position.distance(transform.translation.xy()) >= 34.0 {
            continue;
        }

        match level_state
            .0
            .objectives
            .exit_readiness(&exit.required_objectives)
        {
            ExitReadiness::Ready => {
                if campaign_signal.pending_exit_target.as_ref() != Some(&exit.target) {
                    campaign_signal.pending_exit_target = Some(exit.target.clone());
                    info!("exit target={} is ready", exit.target);
                }
            }
            ExitReadiness::Blocked { missing } => {
                debug!(
                    "exit target={} is locked by objectives {:?}",
                    exit.target, missing
                );
            }
        }
    }
}
