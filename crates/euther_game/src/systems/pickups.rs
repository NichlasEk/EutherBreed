use bevy::prelude::*;
use game_core::{AxisAlignedBox, DoorKind};
use game_core::{ExitReadiness, PickupKind};

use crate::components::{Apothecary, Door, ExitZone, Pickup, Wall};
use crate::resources::{
    ApothecaryVitals, CampaignSignal, ContaminantSpawnTimer, GameNotice, LevelRuntime,
    LocalLevelState, PendingExit,
};

const APOTHECARY_RADIUS: f32 = 22.0;
const PICKUP_RADIUS: f32 = 14.0;

pub fn collect_pickups(
    mut commands: Commands,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    pickup_query: Query<(Entity, &Transform, &Pickup)>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut level_state: ResMut<LocalLevelState>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    mut notice: ResMut<GameNotice>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (entity, transform, pickup) in &pickup_query {
        let distance = apothecary_position.distance(transform.translation.xy());

        if distance > APOTHECARY_RADIUS + PICKUP_RADIUS {
            continue;
        }

        match pickup.kind {
            PickupKind::ReagentRounds(amount) => {
                vitals.0.add_ammo(amount);
                notice.show(format!("Reagent rounds +{amount}"), 1.4);
            }
            PickupKind::MedGel(amount) => {
                vitals.0.heal(amount, 100);
                notice.show(format!("Med-gel +{amount}"), 1.4);
            }
            PickupKind::BioSample => {
                vitals.0.collect_bio_sample();
                notice.show("Bio-sample secured", 1.4);
            }
            PickupKind::SecurityKeycard(ref clearance_id) => {
                level_state.0.grant_clearance(clearance_id.clone());
                if level_runtime.dynamic_spawn_interval_seconds > 0.0 {
                    level_runtime.dynamic_spawn_interval_seconds =
                        level_runtime.dynamic_spawn_interval_seconds.min(3.0);
                    contaminant_timer
                        .0
                        .set_duration(std::time::Duration::from_secs_f32(
                            level_runtime.dynamic_spawn_interval_seconds,
                        ));
                }
                notice.show("Security clearance acquired", 1.6);
            }
            PickupKind::AreaScan => {
                level_state.0.acquire_area_scan();
                notice.show("Area scan acquired - hold Shift for map", 1.8);
            }
        }

        level_state.0.collect_pickup(pickup.id.clone());
        commands.entity(entity).despawn();
    }
}

pub fn unlock_doors(
    mut commands: Commands,
    mut level_state: ResMut<LocalLevelState>,
    mut door_query: Query<(Entity, &mut Door, &mut Sprite), With<Wall>>,
    mut notice: ResMut<GameNotice>,
) {
    if !level_state.is_changed() {
        return;
    }

    for (entity, mut door, mut sprite) in &mut door_query {
        if !door.locked || !door_requirements_met(&door, &level_state) {
            continue;
        }

        door.locked = false;
        level_state.0.unlock_door(door.id.clone());
        sprite.color = match door.kind {
            DoorKind::Bulkhead => Color::srgba(0.55, 0.85, 0.80, 0.42),
            DoorKind::EnergyBarrier => Color::srgba(0.20, 0.95, 1.0, 0.26),
        };
        commands.entity(entity).remove::<Wall>();
        let message = match door.kind {
            DoorKind::Bulkhead => "Door unlocked",
            DoorKind::EnergyBarrier => "Energy barrier disabled",
        };
        notice.show(message, 1.4);
    }
}

fn door_requirements_met(door: &Door, level_state: &LocalLevelState) -> bool {
    let clearance_met =
        door.clearance_id == "open" || level_state.0.has_clearance(&door.clearance_id);
    let objectives_met = door
        .required_objectives
        .iter()
        .all(|objective_id| level_state.0.objectives.is_complete(objective_id));

    clearance_met && objectives_met
}

pub fn report_exit_overlap(
    apothecary_query: Single<&Transform, With<Apothecary>>,
    exit_query: Query<(&Transform, &ExitZone)>,
    level_state: Res<LocalLevelState>,
    mut campaign_signal: ResMut<CampaignSignal>,
    mut notice: ResMut<GameNotice>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (transform, exit) in &exit_query {
        let exit_bounds = AxisAlignedBox::new(transform.translation.xy(), exit.half_extents);
        if !point_inside_expanded_box(apothecary_position, exit_bounds, APOTHECARY_RADIUS) {
            continue;
        }

        match level_state
            .0
            .objectives
            .exit_readiness(&exit.required_objectives)
        {
            ExitReadiness::Ready => {
                if campaign_signal.pending_exit.as_ref().is_none_or(|pending| {
                    pending.target != exit.target || pending.entry_id != exit.entry_id
                }) {
                    campaign_signal.pending_exit = Some(PendingExit {
                        target: exit.target.clone(),
                        entry_id: exit.entry_id.clone(),
                    });
                    info!(
                        "exit target={} entry={} is ready",
                        exit.target, exit.entry_id
                    );
                }
            }
            ExitReadiness::Blocked { missing } => {
                notice.show("Exit locked: objective incomplete", 1.6);
                debug!(
                    "exit target={} is locked by objectives {:?}",
                    exit.target, missing
                );
            }
        }
    }
}

fn point_inside_expanded_box(point: Vec2, area: AxisAlignedBox, expansion: f32) -> bool {
    let min = area.center - area.half_extents - Vec2::splat(expansion);
    let max = area.center + area.half_extents + Vec2::splat(expansion);

    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}
