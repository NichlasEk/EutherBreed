//! Optional noisy access: damaged steel, persistent damage and one-shot ambushes.
use crate::audio_settings::{AudioSettings, play_sfx};
use crate::components::{Door, DoorOpening, EffectLifetime, LevelEntity, Wall};
use crate::resources::LocalLevelState;
use bevy::prelude::*;
use game_core::{DoorBreach, DoorDefinition, LevelState};

#[derive(Component)]
pub struct Breachable {
    pub definition: DoorBreach,
    pub damage: u16,
    pub flash: f32,
}
#[derive(Component)]
pub(crate) struct Scar {
    owner: Entity,
    index: usize,
    size: Vec2,
}
#[derive(Component)]
pub(crate) struct Fragment(Vec2);

pub fn enemy_id(door: &str, index: usize) -> String {
    format!("breach:{door}:{index}")
}

pub fn restore_door(
    commands: &mut Commands,
    assets: &AssetServer,
    owner: Entity,
    door: &DoorDefinition,
    state: &LevelState,
) {
    let Some(definition) = &door.breach else {
        return;
    };
    let damage = state
        .door_damage
        .get(&door.id)
        .copied()
        .unwrap_or(0)
        .min(definition.hits);
    commands.entity(owner).insert(Breachable {
        definition: definition.clone(),
        damage,
        flash: 0.0,
    });
    for index in 0..7 {
        commands.spawn((
            Sprite::from_color(Color::NONE, Vec2::ONE),
            Transform::from_xyz(0.0, 0.0, 1.2),
            ChildOf(owner),
            Scar {
                owner,
                index,
                size: door.half_extents * 2.0,
            },
        ));
    }
    if damage >= definition.hits {
        for (index, position) in definition.alarm_spawns.iter().enumerate() {
            let id = enemy_id(&door.id, index);
            if !state.has_killed_contaminant(&id) {
                crate::setup::spawn_contaminant(commands, assets, Some(id), *position);
            }
        }
    }
}

pub fn hit(
    commands: &mut Commands,
    assets: &AssetServer,
    audio: &AudioSettings,
    state: &mut LocalLevelState,
    entity: Entity,
    door: &mut Door,
    breach: &mut Breachable,
    position: Vec2,
) {
    if door.opened || breach.damage >= breach.definition.hits {
        return;
    }
    breach.damage += 1;
    breach.flash = 0.12;
    state.0.door_damage.insert(door.id.clone(), breach.damage);
    let destroyed = breach.damage == breach.definition.hits;
    play_sfx(
        commands,
        assets,
        audio,
        if destroyed {
            "audio/bulkhead-breach.ogg"
        } else {
            "audio/bulkhead-impact.ogg"
        },
    );
    let count = if destroyed { 16 } else { 5 };
    for index in 0..count {
        let angle = index as f32 * 2.399 + breach.damage as f32;
        let direction = Vec2::from_angle(angle);
        commands.spawn((
            Sprite::from_color(
                if destroyed {
                    Color::srgb(0.65, 0.39, 0.16)
                } else {
                    Color::srgb(1.0, 0.70, 0.25)
                },
                Vec2::new(if destroyed { 9.0 } else { 4.0 }, 2.0),
            ),
            Transform::from_translation(position.extend(24.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            Fragment(
                direction
                    * if destroyed {
                        95.0 + (index % 4) as f32 * 25.0
                    } else {
                        70.0
                    },
            ),
            EffectLifetime(Timer::from_seconds(
                if destroyed { 0.7 } else { 0.18 },
                TimerMode::Once,
            )),
            LevelEntity,
        ));
    }
    if destroyed {
        door.opened = true;
        door.locked = false;
        state.0.unlock_door(&door.id);
        commands
            .entity(entity)
            .remove::<Wall>()
            .remove::<DoorOpening>();
        for (index, position) in breach.definition.alarm_spawns.iter().enumerate() {
            crate::setup::spawn_contaminant(
                commands,
                assets,
                Some(enemy_id(&door.id, index)),
                *position,
            );
        }
    }
}

pub fn animate_breaches(
    time: Res<Time>,
    mut doors: Query<(&Door, &mut Breachable)>,
    mut scars: Query<(&Scar, &mut Sprite, &mut Transform), Without<Fragment>>,
    mut fragments: Query<(&mut Fragment, &mut Transform), Without<Scar>>,
) {
    for (_, mut breach) in &mut doors {
        breach.flash = (breach.flash - time.delta_secs()).max(0.0);
    }
    for (scar, mut sprite, mut transform) in &mut scars {
        let Ok((door, breach)) = doors.get(scar.owner) else {
            continue;
        };
        let broken = breach.damage >= breach.definition.hits;
        let ratio = breach.damage as f32 / breach.definition.hits.max(1) as f32;
        let horizontal = scar.size.x >= scar.size.y;
        let (axis, cross, span, depth) = if horizontal {
            (Vec2::X, Vec2::Y, scar.size.x, scar.size.y)
        } else {
            (Vec2::Y, -Vec2::X, scar.size.y, scar.size.x)
        };
        let n = scar.index as f32;
        let offset = if broken {
            axis * ((n - 3.0) * span / 7.0)
                + cross * (depth * 0.5 + 3.0) * (if scar.index % 2 == 0 { 1.0 } else { -1.0 })
        } else {
            axis * ((n - 3.0) * span / 9.0)
        };
        transform.translation.x = offset.x;
        transform.translation.y = offset.y;
        transform.rotation = Quat::from_rotation_z(axis.to_angle() + 0.6 + (n * 2.1).sin() * 0.4);
        sprite.custom_size = Some(Vec2::new(8.0 + ratio * 6.0, 1.4 + ratio * 1.7));
        sprite.color = if door.opened && !broken {
            Color::NONE
        } else if breach.flash > 0.0 {
            Color::srgb(1.0, 0.68, 0.25)
        } else if broken {
            Color::srgb(0.25, 0.13, 0.065)
        } else {
            Color::srgb(0.48 + ratio * 0.2, 0.23, 0.06)
        };
    }
    for (mut fragment, mut transform) in &mut fragments {
        transform.translation += (fragment.0 * time.delta_secs()).extend(0.0);
        fragment.0 *= (-4.0 * time.delta_secs()).exp();
        transform.rotate_z(time.delta_secs() * 4.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Contaminant, Projectile};
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn scene(state: LevelState) -> (App, Entity, DoorDefinition) {
        let level =
            game_core::LevelDefinition::from_ron_file("../../assets/levels/specimen_archive.ron")
                .unwrap();
        let definition = level
            .doors
            .iter()
            .find(|d| d.id == "reserve_gate")
            .unwrap()
            .clone();
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<AudioSource>()
            .insert_resource(AudioSettings {
                sfx: 0.0,
                ..default()
            })
            .insert_resource(LocalLevelState(state.clone()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .add_systems(Update, crate::systems::move_projectiles);
        let opened = state.has_unlocked_door(&definition.id);
        let owner = app
            .world_mut()
            .spawn((
                Door {
                    id: definition.id.clone(),
                    clearance_id: definition.clearance_id.clone(),
                    locked: !opened,
                    opened,
                    kind: definition.kind,
                    required_objectives: vec![],
                },
                Transform::from_translation(definition.position.extend(0.0)),
                Sprite::from_color(Color::NONE, definition.half_extents * 2.0),
                LevelEntity,
            ))
            .id();
        if !opened {
            app.world_mut().entity_mut(owner).insert(Wall {
                half_extents: definition.half_extents,
            });
        }
        let assets = app.world().resource::<AssetServer>().clone();
        restore_door(
            &mut app.world_mut().commands(),
            &assets,
            owner,
            &definition,
            &state,
        );
        app.world_mut().flush();
        app.update();
        (app, owner, definition)
    }
    fn fire(app: &mut App) {
        app.world_mut().spawn((
            Projectile {
                velocity: Vec2::Y * 2000.0,
                lifetime: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Transform::default(),
            LevelEntity,
        ));
        app.update();
    }
    fn hosts(app: &mut App) -> usize {
        app.world_mut()
            .query::<&Contaminant>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn damage_save_breach_and_killed_ambush_survive_reload_without_duplicates() {
        let (mut app, owner, definition) = scene(LevelState::default());
        for _ in 0..4 {
            fire(&mut app);
        }
        assert_eq!(app.world().get::<Breachable>(owner).unwrap().damage, 4);
        assert!(app.world().get::<Wall>(owner).is_some());
        assert_eq!(hosts(&mut app), 0);
        let saved = ron::to_string(&app.world().resource::<LocalLevelState>().0).unwrap();
        let (mut app, owner, _) = scene(ron::from_str(&saved).unwrap());
        for _ in 4..definition.breach.unwrap().hits {
            fire(&mut app);
        }
        assert!(app.world().get::<Wall>(owner).is_none());
        assert!(app.world().get::<Door>(owner).unwrap().opened);
        assert_eq!(hosts(&mut app), 2);
        fire(&mut app);
        fire(&mut app);
        assert_eq!(hosts(&mut app), 2);
        let mut state = app.world().resource::<LocalLevelState>().0.clone();
        state.kill_contaminant(enemy_id("reserve_gate", 0));
        let (mut app, owner, _) = scene(state);
        assert!(app.world().get::<Wall>(owner).is_none());
        assert_eq!(hosts(&mut app), 1);
        fire(&mut app);
        assert_eq!(hosts(&mut app), 1);
    }

    #[test]
    fn quiet_keycard_opening_does_not_trigger_the_breach_ambush() {
        let mut state = LevelState::default();
        state.grant_clearance("reserve_stock");
        state.unlock_door("reserve_gate");
        let (mut app, owner, _) = scene(state);
        fire(&mut app);
        assert_eq!(app.world().get::<Breachable>(owner).unwrap().damage, 0);
        assert_eq!(hosts(&mut app), 0);
    }

    #[test]
    fn swept_round_stops_at_the_first_solid_wall() {
        let (mut app, owner, _) = scene(LevelState::default());
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 50.0, 0.0),
            Wall {
                half_extents: Vec2::new(80.0, 4.0),
            },
        ));
        fire(&mut app);
        assert_eq!(app.world().get::<Breachable>(owner).unwrap().damage, 0);
        assert_eq!(
            app.world_mut()
                .query::<&Projectile>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
