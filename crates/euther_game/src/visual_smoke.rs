//! Reproducible render check: capture the ward and both door states, then quit.
use crate::{
    AppScreen,
    components::{Door, DoorOpening},
    resources::{ApothecaryVitals, CampaignSignal, PendingExit, SaveSlot},
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

#[derive(Resource)]
struct VisualSmoke {
    directory: String,
    expected_level: String,
    elapsed: f32,
    stage: usize,
}

pub fn configure(app: &mut App, directory: String) {
    if let Some(level) = crate::argument_value("--visual-level") {
        let mut campaign = app
            .world_mut()
            .resource_mut::<crate::resources::CampaignRuntime>();
        let known = campaign.definition.contains_level(&level);
        campaign
            .progress
            .travel_to_known_level(known, &level)
            .expect("known visual level");
    }
    app.world_mut()
        .resource_mut::<CampaignSignal>()
        .exit_lock_active = true;
    let expected_level = app
        .world()
        .resource::<crate::resources::CampaignRuntime>()
        .progress
        .current_level()
        .to_string();
    std::fs::create_dir_all(&directory).expect("create visual smoke output directory");
    app.insert_resource(SaveSlot {
        path: std::path::Path::new(&directory).join("smoke-save.ron"),
    })
    .insert_state(AppScreen::InGame)
    .insert_resource(VisualSmoke {
        directory,
        expected_level,
        elapsed: 0.0,
        stage: 0,
    })
    .add_systems(PostUpdate, overview_camera)
    .add_systems(Startup, fullscreen_review);
    if std::env::args().any(|a| a == "--breach-review") {
        assert_eq!(
            app.world().resource::<VisualSmoke>().expected_level,
            "specimen_archive"
        );
        app.add_systems(PreUpdate, breach_review.after(bevy::input::InputSystems));
    } else if std::env::args().any(|a| a == "--terminal-review") {
        app.add_systems(PreUpdate, terminal_review.after(bevy::input::InputSystems));
    } else {
        app.add_systems(Update, capture.run_if(in_state(AppScreen::InGame)));
    }
}

fn capture(
    mut commands: Commands,
    time: Res<Time>,
    mut smoke: ResMut<VisualSmoke>,
    mut doors: Query<(Entity, &mut Door)>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut exit: MessageWriter<AppExit>,
    mut campaign: ResMut<CampaignSignal>,
    cameras: Query<Entity, With<Camera2d>>,
    hud: Query<Entity, With<crate::components::HudRoot>>,
    map: Res<crate::resources::CurrentLevelMap>,
) {
    smoke.elapsed += time.delta_secs();
    vitals.0.health = 100;
    let threshold = [1.5, 2.0, 2.4, 3.2, 4.0, 5.5, 6.5][smoke.stage.min(6)];
    if smoke.elapsed < threshold {
        return;
    }
    match smoke.stage {
        0 | 2 | 3 | 5 => {
            if smoke.stage == 0 || smoke.stage == 5 {
                assert_eq!(map.level.as_ref().unwrap().name, smoke.expected_level);
            }
            assert_eq!(
                cameras.iter().count(),
                1,
                "one active camera throughout play"
            );
            assert_eq!(
                hud.iter().count(),
                2,
                "exactly two HUD rails throughout play"
            );
            let name = match smoke.stage {
                0 => "ward-closed",
                2 => "ward-opening",
                3 => "ward-open",
                _ => "after-transition",
            };
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("{}/{name}.png", smoke.directory)));
        }
        1 => {
            for (entity, mut door) in &mut doors {
                if !door.opened {
                    door.locked = false;
                    commands.entity(entity).insert(DoorOpening {
                        timer: Timer::from_seconds(0.85, TimerMode::Once),
                    });
                }
            }
        }
        4 => {
            let route = &map.level.as_ref().unwrap().exits[0];
            smoke.expected_level = route.target.clone();
            campaign.pending_exit = Some(PendingExit {
                target: route.target.clone(),
                entry_id: route.entry_id.clone(),
            });
        }
        _ => {
            exit.write(AppExit::Success);
        }
    }
    smoke.stage += 1;
}

fn overview_camera(
    map: Res<crate::resources::CurrentLevelMap>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Some(level) = &map.level else { return };
    for (mut transform, mut projection) in &mut camera {
        transform.translation.x = level.bounds.center.x;
        transform.translation.y = level.bounds.center.y;
        if let Projection::Orthographic(p) = &mut *projection {
            p.scaling_mode = bevy::camera::ScalingMode::AutoMin {
                min_width: level.bounds.half_extents.x * 2.0 + 160.0,
                min_height: level.bounds.half_extents.y * 2.0 + 180.0,
            };
        }
    }
}

fn fullscreen_review(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.mode =
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current);
    }
}

fn terminal_review(
    mut commands: Commands,
    time: Res<Time>,
    mut review: ResMut<VisualSmoke>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut player: Query<&mut Transform, With<crate::components::Apothecary>>,
    mut vitals: ResMut<ApothecaryVitals>,
    local: Res<crate::resources::LocalLevelState>,
    mut next: ResMut<NextState<AppScreen>>,
    mut back: ResMut<crate::audio_settings::SettingsReturn>,
    audio: Res<crate::audio_settings::AudioSettings>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    mouse.reset_all();
    vitals.0.health = 100;
    review.elapsed += time.delta_secs();
    let times = [
        1.5, 2.0, 2.25, 2.9, 3.6, 5.0, 5.2, 5.9, 6.4, 7.3, 8.1, 8.8, 9.2, 9.5, 9.8, 10.3, 10.8,
        11.4,
    ];
    if review.elapsed < times[review.stage.min(times.len() - 1)] {
        return;
    }
    info!("terminal review stage {} audio={:?}", review.stage, *audio);
    let screenshot = match review.stage {
        0 => Some("terminal-idle"),
        2 => Some("terminal-denied"),
        4 => Some("analysis-complete"),
        6 => Some("terminal-used"),
        9 => Some("analysis-restored"),
        11 => Some("audio-settings"),
        15 => Some("audio-adjusted"),
        _ => None,
    };
    if let Some(name) = screenshot {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("{}/{name}.png", review.directory)));
    }
    match review.stage {
        1 => {
            if let Ok(mut p) = player.single_mut() {
                p.translation.x = 75.0;
                p.translation.y = -190.0;
            }
            vitals.0.bio_samples = 0;
            keys.press(KeyCode::KeyE);
        }
        3 => {
            vitals.0.bio_samples = 1;
            keys.press(KeyCode::KeyE);
        }
        4 => assert!(local.0.activated_terminals.contains("ward_lab_analyzer")),
        5 => {
            vitals.0.bio_samples = 0;
            keys.press(KeyCode::KeyE);
        }
        7 => {
            keys.press(KeyCode::F5);
        }
        8 => {
            keys.press(KeyCode::F9);
        }
        9 => assert!(local.0.objectives.is_complete("analyze_contaminant_sample")),
        10 => {
            back.0 = AppScreen::Paused;
            next.set(AppScreen::Settings);
        }
        12 => {
            keys.press(KeyCode::ArrowRight);
        }
        13 => {
            keys.press(KeyCode::Tab);
        }
        14 => {
            keys.press(KeyCode::ArrowLeft);
        }
        15 => {
            assert!((audio.music - 0.37).abs() < 0.001);
            assert!((audio.sfx - 0.75).abs() < 0.001);
        }
        16 => {
            keys.press(KeyCode::Escape);
        }
        17 => {
            exit.write(AppExit::Success);
        }
        _ => (),
    }
    review.stage += 1;
}

fn breach_review(
    mut commands: Commands,
    time: Res<Time>,
    mut review: ResMut<VisualSmoke>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut player: Query<&mut Transform, With<crate::components::Apothecary>>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut local: ResMut<crate::resources::LocalLevelState>,
    doors: Query<(&Door, Option<&crate::components::Wall>)>,
    enemies: Query<(Entity, &crate::components::Contaminant)>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    mouse.reset_all();
    vitals.0.health = 100;
    review.elapsed += time.delta_secs();
    let times = [
        1.0, 1.6, 1.9, 2.2, 2.5, 3.0, 3.6, 4.0, 4.8, 5.1, 5.4, 5.7, 6.0, 6.3, 6.6, 7.0, 7.8, 8.3,
        9.1, 9.8,
    ];
    if review.elapsed < times[review.stage.min(times.len() - 1)] {
        return;
    }
    let stage = review.stage;
    info!(
        "breach review stage {stage}: damage={:?}",
        local.0.door_damage.get("reserve_gate")
    );
    match stage {
        0 => {
            if let Ok(mut p) = player.single_mut() {
                p.translation.x = 0.0;
                p.translation.y = 15.0;
            }
        }
        1..=4 | 9..=14 => {
            commands.spawn((
                Sprite::from_color(Color::srgb(0.7, 1.0, 0.9), Vec2::new(4.0, 18.0)),
                Transform::from_xyz(0.0, 50.0, 20.0),
                crate::components::Projectile {
                    velocity: Vec2::Y * 720.0,
                    lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                },
                crate::components::LevelEntity,
            ));
        }
        5 | 8 => assert_eq!(local.0.door_damage.get("reserve_gate"), Some(&4)),
        6 => keys.press(KeyCode::F5),
        7 | 17 => keys.press(KeyCode::F9),
        15 | 18 => {
            assert_eq!(local.0.door_damage.get("reserve_gate"), Some(&10));
            let (door, wall) = doors.iter().find(|(d, _)| d.id == "reserve_gate").unwrap();
            assert!(door.opened && wall.is_none());
            let count = enemies
                .iter()
                .filter(|(_, e)| e.id.as_deref().is_some_and(|id| id.starts_with("breach:")))
                .count();
            assert_eq!(count, if stage == 15 { 2 } else { 1 });
        }
        16 => {
            let id = crate::breach::enemy_id("reserve_gate", 0);
            for (entity, enemy) in &enemies {
                if enemy.id.as_deref() == Some(&id) {
                    commands.entity(entity).despawn();
                }
            }
            local.0.kill_contaminant(id);
            keys.press(KeyCode::F5);
        }
        _ => {
            exit.write(AppExit::Success);
        }
    }
    if let Some(name) = match stage {
        0 => Some("intact"),
        5 => Some("damaged"),
        8 => Some("damage-restored"),
        15 => Some("breached"),
        18 => Some("breach-restored"),
        _ => None,
    } {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("{}/{name}.png", review.directory)));
    }
    review.stage += 1;
}
