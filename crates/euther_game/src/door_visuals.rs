//! Layered bulkheads and electric fields. All parts belong to the gameplay door,
//! so transitions and save/load remove/reconstruct the entire assembly together.
use bevy::prelude::*;
use game_core::{DoorKind, RuleContext, RuleGate};

use crate::components::{Door, DoorOpening};
use crate::resources::{ApothecaryVitals, LocalLevelState};

#[derive(Component)]
pub struct DoorRejection(pub Timer);
pub fn tick_rejections(
    mut commands: Commands,
    time: Res<Time>,
    mut doors: Query<(Entity, &mut DoorRejection)>,
) {
    for (entity, mut rejection) in &mut doors {
        rejection.0.tick(time.delta());
        if rejection.0.is_finished() {
            commands.entity(entity).remove::<DoorRejection>();
        }
    }
}

const ARC_SEGMENTS: usize = 12;

#[derive(Component)]
pub(crate) struct DoorPart {
    owner: Entity,
    size: Vec2,
    role: PartRole,
}

#[derive(Clone, Copy)]
enum PartRole {
    Leaf {
        side: f32,
    },
    Indicator,
    Field {
        layer: usize,
    },
    Arc {
        strand: usize,
        segment: usize,
        halo: bool,
    },
}

fn axes(size: Vec2) -> (Vec2, Vec2, f32, f32) {
    if size.x >= size.y {
        (Vec2::X, Vec2::Y, size.x, size.y)
    } else {
        (Vec2::Y, -Vec2::X, size.y, size.x)
    }
}

fn part(
    commands: &mut Commands,
    owner: Entity,
    sprite: Sprite,
    position: Vec2,
    z: f32,
    angle: f32,
) -> Entity {
    commands
        .spawn((
            sprite,
            Transform::from_translation(position.extend(z))
                .with_rotation(Quat::from_rotation_z(angle)),
            ChildOf(owner),
        ))
        .id()
}

fn plate(
    commands: &mut Commands,
    owner: Entity,
    position: Vec2,
    size: Vec2,
    z: f32,
    angle: f32,
    color: Color,
) -> Entity {
    part(
        commands,
        owner,
        Sprite::from_color(color, size),
        position,
        z,
        angle,
    )
}

pub fn spawn_door_visuals(
    commands: &mut Commands,
    owner: Entity,
    size: Vec2,
    kind: DoorKind,
    opened: bool,
    texture: Handle<Image>,
) {
    let (axis, cross, span, depth) = axes(size);
    let angle = axis.to_angle();
    // Recessed threshold and fixed steel tracks remain after the door opens.
    plate(
        commands,
        owner,
        Vec2::ZERO,
        Vec2::new(span + 18.0, depth + 18.0),
        -0.8,
        angle,
        Color::srgb(0.025, 0.035, 0.05),
    );
    for side in [-1.0, 1.0] {
        plate(
            commands,
            owner,
            cross * side * (depth * 0.5 + 4.0),
            Vec2::new(span + 24.0, 5.0),
            0.4,
            angle,
            Color::srgb(0.22, 0.29, 0.34),
        );
        plate(
            commands,
            owner,
            cross * side * (depth * 0.5 + 7.0),
            Vec2::new(span + 24.0, 1.0),
            0.5,
            angle,
            Color::srgb(0.40, 0.51, 0.57),
        );
    }
    if kind == DoorKind::Bulkhead {
        for side in [-1.0, 1.0] {
            // The two halves retain their scale; they physically slide into the jambs.
            let mut sprite = Sprite::from_image(texture.clone());
            sprite.rect = Some(Rect::new(
                if side < 0.0 { 0.0 } else { 199.0 },
                0.0,
                if side < 0.0 { 199.0 } else { 398.0 },
                276.0,
            ));
            sprite.custom_size = Some(Vec2::new(span * 0.5, depth));
            let leaf = part(
                commands,
                owner,
                sprite,
                axis * leaf_offset(span, side, if opened { 1.0 } else { 0.0 }),
                0.1,
                angle,
            );
            commands.entity(leaf).insert(DoorPart {
                owner,
                size,
                role: PartRole::Leaf { side },
            });
        }
    } else {
        for layer in 0..4 {
            let field = plate(
                commands,
                owner,
                Vec2::ZERO,
                Vec2::new(span, depth + layer as f32 * 12.0),
                -0.3 + layer as f32 * 0.01,
                angle,
                Color::NONE,
            );
            commands.entity(field).insert(DoorPart {
                owner,
                size,
                role: PartRole::Field { layer },
            });
        }
        for strand in 0..3 {
            for segment in 0..ARC_SEGMENTS {
                for halo in [true, false] {
                    let arc = plate(
                        commands,
                        owner,
                        Vec2::ZERO,
                        Vec2::ONE,
                        if halo { 0.6 } else { 0.7 },
                        0.0,
                        Color::NONE,
                    );
                    commands.entity(arc).insert(DoorPart {
                        owner,
                        size,
                        role: PartRole::Arc {
                            strand,
                            segment,
                            halo,
                        },
                    });
                }
            }
        }
    }
    for side in [-1.0, 1.0] {
        let pos = axis * side * (span * 0.5 + 6.0);
        // Heavy fixed emitters/jambs mask the retracting ends.
        plate(
            commands,
            owner,
            pos,
            Vec2::new(15.0, depth + 24.0),
            0.8,
            angle,
            Color::srgb(0.06, 0.09, 0.12),
        );
        plate(
            commands,
            owner,
            pos,
            Vec2::new(9.0, depth + 17.0),
            0.9,
            angle,
            Color::srgb(0.24, 0.30, 0.34),
        );
        for end in [-1.0, 1.0] {
            plate(
                commands,
                owner,
                pos + cross * end * (depth * 0.5 + 6.0),
                Vec2::new(4.0, 2.0),
                1.0,
                angle,
                Color::srgb(0.76, 0.55, 0.19),
            );
        }
        let indicator = plate(
            commands,
            owner,
            pos,
            Vec2::new(3.0, depth * 0.65),
            1.1,
            angle,
            Color::srgb(0.12, 0.55, 1.0),
        );
        commands.entity(indicator).insert(DoorPart {
            owner,
            size,
            role: PartRole::Indicator,
        });
    }
}

// Crop at the fixed jamb as each leaf slides out; never squash its texture.
fn leaf_offset(span: f32, side: f32, progress: f32) -> f32 {
    side * span * 0.25 * (1.0 + progress)
}

fn arc_point(size: Vec2, strand: usize, index: usize, time: f32) -> Vec2 {
    let (axis, cross, span, depth) = axes(size);
    let u = index as f32 / ARC_SEGMENTS as f32;
    let envelope = (u * std::f32::consts::PI).sin();
    let tick = (time * 18.0).floor();
    let seed = index as f32 * 17.31 + strand as f32 * 43.7 + tick * 7.13;
    let jagged = (seed.sin() * 19.7).sin();
    axis * ((u - 0.5) * span)
        + cross * envelope * ((strand as f32 - 1.0) * depth * 0.24 + jagged * depth * 0.21)
}

pub fn animate_door_visuals(
    time: Res<Time>,
    state: Res<LocalLevelState>,
    vitals: Res<ApothecaryVitals>,
    doors: Query<(&Door, Option<&DoorOpening>, Option<&DoorRejection>)>,
    mut parts: Query<(&DoorPart, &mut Transform, &mut Sprite)>,
) {
    let t = time.elapsed_secs();
    for (part, mut transform, mut sprite) in &mut parts {
        let Ok((door, opening, rejection)) = doors.get(part.owner) else {
            continue;
        };
        let progress = if door.opened {
            1.0
        } else {
            opening.map_or(0.0, |o| o.timer.fraction())
        };
        let eased = progress * progress * (3.0 - 2.0 * progress);
        let strength = 1.0 - eased;
        let (axis, _, span, depth) = axes(part.size);
        let pulse = 0.78 + 0.22 * (t * 5.0 + span).sin();
        match part.role {
            PartRole::Leaf { side } => {
                sprite.custom_size = Some(Vec2::new((span * 0.5 * strength).max(0.001), depth));
                sprite.rect = Some(if side < 0.0 {
                    Rect::new(199.0 * eased, 0.0, 199.0, 276.0)
                } else {
                    Rect::new(199.0, 0.0, 398.0 - 199.0 * eased, 276.0)
                });
                sprite.color = if door.opened {
                    Color::NONE
                } else {
                    Color::WHITE
                };
                let position = axis * leaf_offset(span, side, eased);
                transform.translation.x = position.x;
                transform.translation.y = position.y;
            }
            PartRole::Indicator => {
                if let Some(rejection) = rejection {
                    sprite.color = Color::srgba(
                        1.0,
                        0.18,
                        0.04,
                        0.55 + 0.45 * (rejection.0.elapsed_secs() * 24.0).sin().abs(),
                    );
                    continue;
                }
                let ready = door.opened
                    || opening.is_some()
                    || state.0.has_unlocked_door(&door.id)
                    || RuleGate::for_door(&door.clearance_id, &door.required_objectives).is_open(
                        RuleContext {
                            level_state: &state.0,
                            vitals: &vitals.0,
                        },
                    );
                sprite.color = if ready {
                    Color::srgb(0.15, 0.85, 0.52)
                } else if door.kind == DoorKind::EnergyBarrier {
                    Color::srgba(0.18, 0.58, 1.0, pulse)
                } else {
                    Color::srgb(0.95, 0.34, 0.09)
                };
            }
            PartRole::Field { layer } => {
                sprite.color = Color::srgba(
                    0.035,
                    0.26,
                    1.0,
                    strength * pulse * (0.16 / (layer + 1) as f32),
                );
                sprite.custom_size = Some(Vec2::new(
                    span,
                    (depth + layer as f32 * 12.0) * (0.6 + strength * 0.4),
                ));
            }
            PartRole::Arc {
                strand,
                segment,
                halo,
            } => {
                let a = arc_point(part.size, strand, segment, t);
                let b = arc_point(part.size, strand, segment + 1, t);
                let center = (a + b) * 0.5;
                transform.translation.x = center.x;
                transform.translation.y = center.y;
                transform.rotation = Quat::from_rotation_z((b - a).to_angle());
                sprite.custom_size =
                    Some(Vec2::new(a.distance(b) + 0.6, if halo { 5.5 } else { 1.1 }));
                sprite.color = if halo {
                    Color::srgba(0.04, 0.33, 1.0, strength * 0.24)
                } else {
                    Color::srgba(0.48, 0.84, 1.0, strength * pulse)
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Apothecary, LevelEntity, Wall};
    use crate::resources::GameNotice;
    use crate::systems::{unlock_doors, update_door_openings};
    use std::time::Duration;

    fn scene(kind: DoorKind, opened: bool, size: Vec2) -> (App, Entity) {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<LocalLevelState>()
            .init_resource::<GameNotice>()
            .insert_resource(ApothecaryVitals(game_core::ApothecaryVitals::new(
                100, 48, 0,
            )))
            .add_systems(
                Update,
                (unlock_doors, update_door_openings, animate_door_visuals).chain(),
            );
        app.world_mut().spawn((Apothecary, Transform::default()));
        let owner = app
            .world_mut()
            .spawn((
                Door {
                    id: "test".into(),
                    clearance_id: "blue".into(),
                    locked: !opened,
                    opened,
                    kind,
                    required_objectives: vec![],
                },
                Sprite::from_color(Color::NONE, size),
                Transform::default(),
                LevelEntity,
            ))
            .id();
        if !opened {
            app.world_mut().entity_mut(owner).insert(Wall {
                half_extents: size * 0.5,
            });
        }
        spawn_door_visuals(
            &mut app.world_mut().commands(),
            owner,
            size,
            kind,
            opened,
            Handle::default(),
        );
        app.world_mut().flush();
        (app, owner)
    }

    #[test]
    fn locked_door_blocks_until_opening_finishes_in_both_orientations() {
        for kind in [DoorKind::Bulkhead, DoorKind::EnergyBarrier] {
            for size in [Vec2::new(96.0, 24.0), Vec2::new(24.0, 96.0)] {
                let (mut app, owner) = scene(kind, false, size);
                app.update();
                assert!(app.world().get::<Wall>(owner).is_some());
                assert!(app.world().get::<DoorOpening>(owner).is_none());
                app.world_mut()
                    .resource_mut::<LocalLevelState>()
                    .0
                    .grant_clearance("blue");
                app.world_mut()
                    .resource_mut::<Time>()
                    .advance_by(Duration::from_secs_f32(0.4));
                app.update();
                assert!(app.world().get::<DoorOpening>(owner).is_some());
                assert_eq!(
                    app.world().get::<Wall>(owner).unwrap().half_extents,
                    size * 0.5
                );
                app.world_mut()
                    .resource_mut::<Time>()
                    .advance_by(Duration::from_secs_f32(0.5));
                app.update();
                assert!(app.world().get::<Wall>(owner).is_none());
                assert!(app.world().get::<Door>(owner).unwrap().opened);
                assert!(
                    app.world()
                        .resource::<LocalLevelState>()
                        .0
                        .has_unlocked_door("test")
                );
                let world = app.world_mut();
                for (part, sprite) in world.query::<(&DoorPart, &Sprite)>().iter(world) {
                    if matches!(
                        part.role,
                        PartRole::Leaf { .. } | PartRole::Field { .. } | PartRole::Arc { .. }
                    ) {
                        assert_eq!(sprite.color.alpha(), 0.0);
                    }
                }
            }
        }
    }

    #[test]
    fn restored_open_door_has_no_field_and_children_are_cleaned_up() {
        for kind in [DoorKind::Bulkhead, DoorKind::EnergyBarrier] {
            let (mut app, owner) = scene(kind, true, Vec2::new(96.0, 24.0));
            app.update();
            assert!(app.world().get::<Wall>(owner).is_none());
            let world = app.world_mut();
            for (part, sprite) in world.query::<(&DoorPart, &Sprite)>().iter(world) {
                if matches!(
                    part.role,
                    PartRole::Leaf { .. } | PartRole::Field { .. } | PartRole::Arc { .. }
                ) {
                    assert_eq!(sprite.color.alpha(), 0.0);
                }
            }
            world.despawn(owner);
            assert_eq!(world.query::<&DoorPart>().iter(world).count(), 0);
        }
    }

    #[test]
    fn objective_field_releases_remotely_after_analysis() {
        let (mut app, owner) = scene(DoorKind::EnergyBarrier, false, Vec2::new(24.0, 96.0));
        app.world_mut()
            .get_mut::<Transform>(owner)
            .unwrap()
            .translation
            .x = 500.0;
        {
            let mut door = app.world_mut().get_mut::<Door>(owner).unwrap();
            door.clearance_id = "open".into();
            door.required_objectives = vec!["analysis".into()];
        }
        app.update();
        assert!(app.world().get::<DoorOpening>(owner).is_none());
        app.world_mut()
            .resource_mut::<LocalLevelState>()
            .0
            .complete_objective("analysis");
        app.update();
        assert!(app.world().get::<DoorOpening>(owner).is_some());
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.update();
        assert!(app.world().get::<Wall>(owner).is_none());
    }

    #[test]
    fn electric_strands_share_emitter_endpoints_and_change_over_time() {
        for size in [Vec2::new(96.0, 24.0), Vec2::new(24.0, 96.0)] {
            let (axis, _, span, _) = axes(size);
            for strand in 0..3 {
                assert!(arc_point(size, strand, 0, 1.0).distance(-axis * span * 0.5) < 0.001);
                assert!(
                    arc_point(size, strand, ARC_SEGMENTS, 2.0).distance(axis * span * 0.5) < 0.001
                );
                assert_ne!(
                    arc_point(size, strand, 5, 1.0),
                    arc_point(size, strand, 5, 2.0)
                );
            }
        }
    }
}
