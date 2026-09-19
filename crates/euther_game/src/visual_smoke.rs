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
    elapsed: f32,
    stage: usize,
}

pub fn configure(app: &mut App, directory: String) {
    std::fs::create_dir_all(&directory).expect("create visual smoke output directory");
    app.insert_resource(SaveSlot {
        path: std::path::Path::new(&directory).join("smoke-save.ron"),
    })
    .insert_state(AppScreen::InGame)
    .insert_resource(VisualSmoke {
        directory,
        elapsed: 0.0,
        stage: 0,
    })
    .add_systems(Update, capture.run_if(in_state(AppScreen::InGame)));
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
) {
    smoke.elapsed += time.delta_secs();
    vitals.0.health = 100;
    let threshold = [1.5, 2.0, 2.4, 3.2, 4.0, 5.5, 6.5][smoke.stage.min(6)];
    if smoke.elapsed < threshold {
        return;
    }
    match smoke.stage {
        0 | 2 | 3 | 5 => {
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
                if matches!(
                    door.id.as_str(),
                    "ward_triage_door" | "ward_quarantine_green_door"
                ) {
                    door.locked = false;
                    commands.entity(entity).insert(DoorOpening {
                        timer: Timer::from_seconds(0.85, TimerMode::Once),
                    });
                }
            }
        }
        4 => {
            campaign.pending_exit = Some(PendingExit {
                target: "lab_access_corridor".into(),
                entry_id: "from_quarantine_ward".into(),
            });
        }
        _ => {
            exit.write(AppExit::Success);
        }
    }
    smoke.stage += 1;
}
