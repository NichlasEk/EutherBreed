//! Persistent ACE-Step score with threat hysteresis and smooth volume changes.
use crate::{
    AppScreen,
    components::{Apothecary, Contaminant, Wall},
    resources::{ApothecaryVitals, GameNotice},
};
use bevy::{audio::Volume, prelude::*};

#[derive(Component)]
pub(crate) struct MusicLayer {
    combat: bool,
    volume: f32,
}

#[derive(Resource)]
pub(crate) struct MusicMix {
    threat_hold: f32,
}
impl Default for MusicMix {
    fn default() -> Self {
        Self { threat_hold: 0.0 }
    }
}

pub fn start_music(mut commands: Commands, assets: Res<AssetServer>) {
    for (path, combat) in [
        ("music/abyssal-circuit.ogg", false),
        ("music/quarantine-pulse.ogg", true),
    ] {
        commands.spawn((
            AudioPlayer::new(assets.load(path)),
            PlaybackSettings {
                volume: Volume::SILENT,
                ..PlaybackSettings::LOOP
            },
            MusicLayer {
                combat,
                volume: 0.0,
            },
        ));
    }
}

fn clear_sight(from: Vec2, to: Vec2, center: Vec2, half: Vec2) -> bool {
    // Slab intersection, including closed doors. An enemy behind a wall should
    // not keep the combat score active simply because it is nearby.
    let delta = to - from;
    let mut enter: f32 = 0.0;
    let mut leave: f32 = 1.0;
    for axis in 0..2 {
        let low = center[axis] - half[axis];
        let high = center[axis] + half[axis];
        if delta[axis].abs() < 0.0001 {
            if from[axis] < low || from[axis] > high {
                return true;
            }
        } else {
            let a = (low - from[axis]) / delta[axis];
            let b = (high - from[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter > leave {
                return true;
            }
        }
    }
    false
}

pub fn update_music(
    time: Res<Time>,
    state: Res<State<AppScreen>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mix: ResMut<MusicMix>,
    mut audio: ResMut<crate::audio_settings::AudioSettings>,
    mut notice: ResMut<GameNotice>,
    vitals: Res<ApothecaryVitals>,
    player: Query<&Transform, With<Apothecary>>,
    enemies: Query<&Transform, With<Contaminant>>,
    walls: Query<(&Transform, &Wall)>,
    mut music: Query<(&mut MusicLayer, &mut AudioSink)>,
) {
    if keys.just_pressed(KeyCode::F6) {
        audio.music_muted = !audio.music_muted;
    }
    if keys.just_pressed(KeyCode::F7) {
        audio.music = (audio.music - 0.08).max(0.0);
    }
    if keys.just_pressed(KeyCode::F8) {
        audio.music = (audio.music + 0.08).min(1.0);
    }
    if [KeyCode::F6, KeyCode::F7, KeyCode::F8]
        .iter()
        .any(|key| keys.just_pressed(*key))
    {
        notice.show(
            if audio.music_muted {
                "Music off".into()
            } else {
                format!("Music {}%", (audio.music * 100.0).round() as i32)
            },
            2.0,
        );
    }
    let playing = *state.get() == AppScreen::InGame && vitals.0.health > 0;
    let threatened = playing
        && player.single().is_ok_and(|p| {
            enemies.iter().any(|e| {
                p.translation.xy().distance(e.translation.xy()) < 230.0
                    && walls.iter().all(|(t, w)| {
                        clear_sight(
                            p.translation.xy(),
                            e.translation.xy(),
                            t.translation.xy(),
                            w.half_extents,
                        )
                    })
            })
        });
    if threatened {
        mix.threat_hold = 6.0;
    } else {
        mix.threat_hold = (mix.threat_hold - time.delta_secs()).max(0.0);
    }
    let combat = playing && mix.threat_hold > 0.0;
    for (mut layer, mut sink) in &mut music {
        let active = layer.combat == combat;
        let target = if audio.music_muted || !active {
            0.0
        } else {
            audio.music * if playing { 1.0 } else { 0.35 }
        };
        // Fade to silence before the other rhythm becomes dominant.
        let rate = if target < layer.volume { 2.5 } else { 0.7 };
        layer.volume += (target - layer.volume) * (1.0 - (-time.delta_secs() * rate).exp());
        sink.set_volume(Volume::Linear(layer.volume));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn music_threat_visibility_respects_walls_and_open_passages() {
        assert!(!clear_sight(
            Vec2::new(-50.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::ZERO,
            Vec2::new(5.0, 40.0)
        ));
        assert!(clear_sight(
            Vec2::new(-50.0, 50.0),
            Vec2::new(50.0, 50.0),
            Vec2::ZERO,
            Vec2::new(5.0, 40.0)
        ));
        assert!(clear_sight(
            Vec2::new(-50.0, 0.0),
            Vec2::new(-25.0, 0.0),
            Vec2::ZERO,
            Vec2::new(5.0, 40.0)
        ));
    }
}
