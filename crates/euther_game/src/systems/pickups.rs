use bevy::prelude::*;
use game_core::PickupKind;

use crate::components::{Apothecary, Door, ExitZone, Pickup, Wall};
use crate::resources::{AccessInventory, ApothecaryVitals, CampaignSignal, ObjectiveState};

const APOTHECARY_RADIUS: f32 = 22.0;
const PICKUP_RADIUS: f32 = 14.0;

pub fn collect_pickups(
    mut commands: Commands,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    pickup_query: Query<(Entity, &Transform, &Pickup)>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut access_inventory: ResMut<AccessInventory>,
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
                access_inventory.clearances.insert(clearance_id.clone());
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn unlock_doors(
    mut commands: Commands,
    access_inventory: Res<AccessInventory>,
    mut door_query: Query<(Entity, &mut Door, &mut Sprite), With<Wall>>,
) {
    if !access_inventory.is_changed() {
        return;
    }

    for (entity, mut door, mut sprite) in &mut door_query {
        if !door.locked || !access_inventory.clearances.contains(&door.clearance_id) {
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
    objective_state: Res<ObjectiveState>,
    mut campaign_signal: ResMut<CampaignSignal>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (transform, exit) in &exit_query {
        if apothecary_position.distance(transform.translation.xy()) >= 34.0 {
            continue;
        }

        let requirements_met = exit
            .required_objectives
            .iter()
            .all(|objective_id| objective_state.completed.contains(objective_id));

        if requirements_met {
            if campaign_signal.pending_exit_target.as_ref() != Some(&exit.target) {
                campaign_signal.pending_exit_target = Some(exit.target.clone());
                info!("exit target={} is ready", exit.target);
            }
        } else {
            debug!("exit target={} is locked by objectives", exit.target);
        }
    }
}
