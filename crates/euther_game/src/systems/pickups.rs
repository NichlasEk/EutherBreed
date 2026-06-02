use bevy::prelude::*;
use game_core::{AxisAlignedBox, DoorKind};
use game_core::{PickupKind, RuleContext, RuleGate, RuleGateStatus};

use crate::components::{
    Apothecary, Door, DoorOpening, DoorOpeningEffect, ExitZone, LevelEntity, Pickup, Wall,
};
use crate::resources::{
    ApothecaryVitals, CampaignSignal, ContaminantSpawnTimer, GameNotice, LevelRuntime,
    LocalLevelState, PendingExit,
};

const APOTHECARY_RADIUS: f32 = 22.0;
const PICKUP_RADIUS: f32 = 14.0;
const DOOR_OPEN_SECONDS: f32 = 0.42;
const DOOR_TRIGGER_PADDING: f32 = APOTHECARY_RADIUS + 24.0;

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
    apothecary_query: Single<&Transform, With<Apothecary>>,
    vitals: Res<ApothecaryVitals>,
    mut level_state: ResMut<LocalLevelState>,
    mut door_query: Query<
        (Entity, &Transform, &mut Door, &Sprite),
        (With<Wall>, Without<DoorOpening>),
    >,
    mut notice: ResMut<GameNotice>,
) {
    let apothecary_position = apothecary_query.translation.xy();

    for (entity, transform, mut door, sprite) in &mut door_query {
        if door.opened
            || !(level_state.0.has_unlocked_door(&door.id)
                || door_requirements_met(&door, &level_state, &vitals))
        {
            continue;
        }

        let door_half_extents = sprite.custom_size.unwrap_or(Vec2::splat(32.0)) * 0.5;
        let trigger_bounds = AxisAlignedBox::new(transform.translation.xy(), door_half_extents);
        if !point_inside_expanded_box(apothecary_position, trigger_bounds, DOOR_TRIGGER_PADDING) {
            continue;
        }

        door.locked = false;
        level_state.0.unlock_door(door.id.clone());
        let door_size = sprite.custom_size.unwrap_or(Vec2::splat(32.0));
        spawn_door_opening_effects(
            &mut commands,
            transform.translation.xy(),
            door_size,
            door.kind,
        );
        commands.entity(entity).insert(DoorOpening {
            timer: Timer::from_seconds(DOOR_OPEN_SECONDS, TimerMode::Once),
            original_size: door_size,
        });
        let message = match door.kind {
            DoorKind::Bulkhead => "Door opening",
            DoorKind::EnergyBarrier => "Energy barrier collapsing",
        };
        notice.show(message, 1.4);
    }
}

pub fn update_door_openings(
    mut commands: Commands,
    time: Res<Time>,
    mut door_query: Query<
        (Entity, &mut Door, &mut DoorOpening, &mut Sprite),
        Without<DoorOpeningEffect>,
    >,
    mut effect_query: Query<
        (Entity, &mut DoorOpeningEffect, &mut Transform, &mut Sprite),
        Without<DoorOpening>,
    >,
) {
    for (entity, mut door, mut opening, mut sprite) in &mut door_query {
        opening.timer.tick(time.delta());
        let progress = opening.timer.fraction();
        let eased = ease_out_cubic(progress);
        let collapse = (1.0 - eased).max(0.05);
        let mut size = opening.original_size;

        if opening.original_size.x >= opening.original_size.y {
            size.x = opening.original_size.x * collapse;
        } else {
            size.y = opening.original_size.y * collapse;
        }

        sprite.custom_size = Some(size);
        sprite.color = door_opening_color(door.kind, eased);

        if opening.timer.is_finished() {
            door.opened = true;
            sprite.custom_size = Some(opening.original_size);
            sprite.color = door_open_color(door.kind);
            commands.entity(entity).remove::<Wall>();
            commands.entity(entity).remove::<DoorOpening>();
        }
    }

    for (entity, mut effect, mut transform, mut sprite) in &mut effect_query {
        effect.timer.tick(time.delta());
        let progress = effect.timer.fraction();
        let eased = ease_out_cubic(progress);
        let shimmer = (progress * std::f32::consts::TAU * 4.0).sin().abs();

        let position = effect.origin + effect.slide * eased;
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        sprite.custom_size = Some(effect.base_size * (1.0 + shimmer * 0.05));
        sprite.color = effect
            .base_color
            .with_alpha((1.0 - progress).powf(1.35).clamp(0.0, 1.0));

        if effect.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_door_opening_effects(commands: &mut Commands, center: Vec2, size: Vec2, kind: DoorKind) {
    match kind {
        DoorKind::Bulkhead => spawn_bulkhead_opening_effects(commands, center, size),
        DoorKind::EnergyBarrier => spawn_energy_barrier_opening_effects(commands, center, size),
    }
}

fn spawn_bulkhead_opening_effects(commands: &mut Commands, center: Vec2, size: Vec2) {
    let horizontal = size.x >= size.y;
    let axis = if horizontal { Vec2::X } else { Vec2::Y };
    let cross = if horizontal { Vec2::Y } else { Vec2::X };
    let panel_size = if horizontal {
        Vec2::new((size.x * 0.46).max(8.0), size.y.max(8.0))
    } else {
        Vec2::new(size.x.max(8.0), (size.y * 0.46).max(8.0))
    };
    let panel_offset = axis
        * (if horizontal {
            panel_size.x
        } else {
            panel_size.y
        } * 0.52);
    let slide = axis * (if horizontal { size.x } else { size.y } * 0.48 + 12.0);

    spawn_opening_effect(
        commands,
        center - panel_offset,
        -slide,
        panel_size,
        Color::srgba(0.34, 0.58, 0.60, 0.88),
        0.46,
        -2.2,
    );
    spawn_opening_effect(
        commands,
        center + panel_offset,
        slide,
        panel_size,
        Color::srgba(0.28, 0.48, 0.50, 0.88),
        0.46,
        -2.2,
    );
    spawn_opening_effect(
        commands,
        center,
        Vec2::ZERO,
        if horizontal {
            Vec2::new(5.0, size.y + 22.0)
        } else {
            Vec2::new(size.x + 22.0, 5.0)
        },
        Color::srgba(0.28, 1.0, 0.92, 0.76),
        0.34,
        -2.0,
    );
    spawn_opening_effect(
        commands,
        center + cross * 2.0,
        cross * 5.0,
        if horizontal {
            Vec2::new(size.x + 18.0, 3.0)
        } else {
            Vec2::new(3.0, size.y + 18.0)
        },
        Color::srgba(0.96, 0.72, 0.30, 0.54),
        0.28,
        -1.9,
    );
}

fn spawn_energy_barrier_opening_effects(commands: &mut Commands, center: Vec2, size: Vec2) {
    let horizontal = size.x >= size.y;
    let axis = if horizontal { Vec2::X } else { Vec2::Y };
    let cross = if horizontal { Vec2::Y } else { Vec2::X };
    let long_size = if horizontal {
        Vec2::new(size.x + 28.0, 4.0)
    } else {
        Vec2::new(4.0, size.y + 28.0)
    };
    let beam_size = if horizontal {
        Vec2::new(4.0, size.y + 30.0)
    } else {
        Vec2::new(size.x + 30.0, 4.0)
    };

    for index in 0..4 {
        let offset = (index as f32 - 1.5) * 7.0;
        let slide = cross * offset * 0.85 + axis * ((index as f32 - 1.5) * 8.0);
        spawn_opening_effect(
            commands,
            center + cross * offset,
            slide,
            long_size,
            if index % 2 == 0 {
                Color::srgba(0.86, 0.16, 1.0, 0.68)
            } else {
                Color::srgba(0.12, 0.98, 1.0, 0.58)
            },
            0.38,
            -1.8,
        );
    }

    spawn_opening_effect(
        commands,
        center,
        Vec2::ZERO,
        beam_size,
        Color::srgba(0.98, 0.30, 1.0, 0.84),
        0.34,
        -1.7,
    );
}

fn spawn_opening_effect(
    commands: &mut Commands,
    origin: Vec2,
    slide: Vec2,
    size: Vec2,
    color: Color,
    seconds: f32,
    z: f32,
) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(origin.x, origin.y, z),
        DoorOpeningEffect {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            origin,
            slide,
            base_size: size,
            base_color: color,
        },
        LevelEntity,
    ));
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn door_requirements_met(
    door: &Door,
    level_state: &LocalLevelState,
    vitals: &ApothecaryVitals,
) -> bool {
    RuleGate::for_door(&door.clearance_id, &door.required_objectives).is_open(RuleContext {
        level_state: &level_state.0,
        vitals: &vitals.0,
    })
}

fn door_opening_color(kind: DoorKind, progress: f32) -> Color {
    match kind {
        DoorKind::Bulkhead => Color::srgba(0.90, 1.0, 0.94, 1.0 - progress * 0.55),
        DoorKind::EnergyBarrier => {
            Color::srgba(0.85, 0.25 + progress * 0.70, 1.0, 1.0 - progress * 0.70)
        }
    }
}

fn door_open_color(kind: DoorKind) -> Color {
    match kind {
        DoorKind::Bulkhead => Color::srgba(0.55, 0.85, 0.80, 0.42),
        DoorKind::EnergyBarrier => Color::srgba(0.20, 0.95, 1.0, 0.26),
    }
}

pub fn report_exit_overlap(
    mut commands: Commands,
    apothecary_query: Single<&Transform, With<Apothecary>>,
    exit_query: Query<(&Transform, &ExitZone)>,
    level_state: Res<LocalLevelState>,
    vitals: Res<ApothecaryVitals>,
    mut campaign_signal: ResMut<CampaignSignal>,
    mut notice: ResMut<GameNotice>,
) {
    let apothecary_position = apothecary_query.translation.xy();
    let overlapping_exit = exit_query.iter().any(|(transform, exit)| {
        let exit_bounds = AxisAlignedBox::new(transform.translation.xy(), exit.half_extents);
        point_inside_expanded_box(apothecary_position, exit_bounds, APOTHECARY_RADIUS)
    });

    if !overlapping_exit {
        campaign_signal.exit_lock_active = false;
        return;
    }

    if campaign_signal.exit_lock_active {
        return;
    }

    for (transform, exit) in &exit_query {
        let exit_bounds = AxisAlignedBox::new(transform.translation.xy(), exit.half_extents);
        if !point_inside_expanded_box(apothecary_position, exit_bounds, APOTHECARY_RADIUS) {
            continue;
        }

        match RuleGate::for_objectives(&exit.required_objectives).evaluate(RuleContext {
            level_state: &level_state.0,
            vitals: &vitals.0,
        }) {
            RuleGateStatus::Open => {
                if campaign_signal.pending_exit.as_ref().is_none_or(|pending| {
                    pending.target != exit.target || pending.entry_id != exit.entry_id
                }) {
                    campaign_signal.pending_exit = Some(PendingExit {
                        target: exit.target.clone(),
                        entry_id: exit.entry_id.clone(),
                    });
                    campaign_signal.exit_lock_active = true;
                    spawn_exit_transit_effect(&mut commands, transform.translation.xy());
                    notice.show("Transit engaged", 0.8);
                    info!(
                        "exit target={} entry={} is ready",
                        exit.target, exit.entry_id
                    );
                }
            }
            RuleGateStatus::Blocked { missing } => {
                notice.show("Exit locked: objective incomplete", 1.6);
                debug!(
                    "exit target={} is locked by objectives {:?}",
                    exit.target, missing
                );
            }
        }
    }
}

fn spawn_exit_transit_effect(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgba(0.18, 1.0, 0.92, 0.60), Vec2::new(66.0, 116.0)),
        Transform::from_xyz(position.x, position.y, 7.0),
        crate::components::EffectLifetime(Timer::from_seconds(0.32, TimerMode::Once)),
        LevelEntity,
    ));
}

fn point_inside_expanded_box(point: Vec2, area: AxisAlignedBox, expansion: f32) -> bool {
    let min = area.center - area.half_extents - Vec2::splat(expansion);
    let max = area.center + area.half_extents + Vec2::splat(expansion);

    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}
