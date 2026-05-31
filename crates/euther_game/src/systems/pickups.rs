use bevy::prelude::*;
use game_core::PickupKind;

use crate::components::{Apothecary, ExitZone, Pickup};
use crate::resources::ApothecaryVitals;

const APOTHECARY_RADIUS: f32 = 22.0;
const PICKUP_RADIUS: f32 = 14.0;

pub fn collect_pickups(
    mut commands: Commands,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    pickup_query: Query<(Entity, &Transform, &Pickup)>,
    mut vitals: ResMut<ApothecaryVitals>,
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
        }

        commands.entity(entity).despawn();
    }
}

pub fn report_exit_overlap(
    apothecary_query: Single<&Transform, With<Apothecary>>,
    exit_query: Query<(&Transform, &ExitZone)>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (transform, exit) in &exit_query {
        if apothecary_position.distance(transform.translation.xy()) < 34.0 {
            debug!("apothecary reached exit target={}", exit.target);
        }
    }
}
