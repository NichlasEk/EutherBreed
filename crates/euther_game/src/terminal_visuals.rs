use crate::{
    components::Terminal,
    resources::{CurrentLevelMap, LocalLevelState},
};
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Response {
    Success,
    Used,
    Denied,
}
#[derive(Component)]
pub struct TerminalPulse {
    pub response: Response,
    pub timer: Timer,
}
#[derive(Component)]
pub(crate) struct TerminalPart {
    owner: Entity,
    role: Part,
}
#[derive(Clone, Copy)]
enum Part {
    Screen,
    Trace(usize),
    Sample,
    Glow,
}
#[derive(Component)]
pub struct RoomLight {
    pub base: Color,
}
#[derive(Component)]
pub struct Ventilation;

pub fn spawn_parts(commands: &mut Commands, owner: Entity) {
    let mut add = |position: Vec2, size: Vec2, color: Color, role: Part, z: f32| {
        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(position.x, position.y, z),
            ChildOf(owner),
            TerminalPart { owner, role },
        ));
    };
    add(
        Vec2::new(0.0, 6.0),
        Vec2::new(16.0, 10.0),
        Color::srgb(0.01, 0.07, 0.09),
        Part::Screen,
        0.1,
    );
    for index in 0..6 {
        add(
            Vec2::new(index as f32 * 2.3 - 5.75, 6.0),
            Vec2::new(1.1, 3.0),
            Color::srgb(0.1, 0.8, 0.9),
            Part::Trace(index),
            0.2,
        );
    }
    add(
        Vec2::new(12.0, -6.0),
        Vec2::new(2.6, 5.5),
        Color::srgb(0.5, 0.14, 0.7),
        Part::Sample,
        0.2,
    );
    add(
        Vec2::new(0.0, 6.0),
        Vec2::new(25.0, 18.0),
        Color::NONE,
        Part::Glow,
        0.0,
    );
}
pub fn respond(commands: &mut Commands, entity: Entity, response: Response) {
    commands.entity(entity).insert(TerminalPulse {
        response,
        timer: Timer::from_seconds(
            if response == Response::Success {
                1.4
            } else {
                0.48
            },
            TimerMode::Once,
        ),
    });
}
pub fn tick_responses(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(Entity, &mut TerminalPulse)>,
) {
    for (entity, mut pulse) in &mut pulses {
        pulse.timer.tick(time.delta());
        if pulse.timer.is_finished() {
            commands.entity(entity).remove::<TerminalPulse>();
        }
    }
}
fn resolved(map: &CurrentLevelMap, state: &LocalLevelState) -> bool {
    map.level.as_ref().is_some_and(|level| {
        let required: Vec<_> = level.objectives.iter().filter(|o| o.required).collect();
        !required.is_empty()
            && required
                .iter()
                .all(|o| state.0.objectives.is_complete(&o.id))
    })
}
pub fn update_terminal_art(
    time: Res<Time>,
    state: Res<LocalLevelState>,
    map: Res<CurrentLevelMap>,
    terminals: Query<(&Terminal, Option<&TerminalPulse>)>,
    mut parts: Query<(&TerminalPart, &mut Sprite), Without<RoomLight>>,
    mut lights: Query<(&RoomLight, &mut Sprite), Without<TerminalPart>>,
) {
    let t = time.elapsed_secs();
    let success_pulse = terminals
        .iter()
        .filter_map(|(_, p)| p)
        .filter(|p| p.response == Response::Success)
        .map(|p| (p.timer.fraction() * std::f32::consts::PI).sin())
        .fold(0.0_f32, f32::max);
    for (part, mut sprite) in &mut parts {
        let Ok((terminal, pulse)) = terminals.get(part.owner) else {
            continue;
        };
        let used = state.0.activated_terminals.contains(&terminal.id);
        let blink = pulse.map_or(0.0, |p| {
            (p.timer.elapsed_secs() * std::f32::consts::TAU * 4.0)
                .sin()
                .max(0.0)
                * (1.0 - p.timer.fraction())
        });
        let color = match pulse.map(|p| p.response) {
            Some(Response::Denied) => Color::srgb(0.95, 0.22, 0.06),
            Some(Response::Used) => Color::srgb(0.20, 0.40, 0.43),
            _ if used => Color::srgb(0.28, 0.84, 0.45),
            _ => Color::srgb(0.08, 0.66, 0.85),
        };
        match part.role {
            Part::Screen => sprite.color = color.with_alpha(0.22 + blink * 0.20),
            Part::Sample => {
                sprite.color = if used {
                    Color::srgb(0.35, 0.90, 0.52)
                } else {
                    Color::srgb(0.58, 0.16, 0.75)
                }
            }
            Part::Glow => {
                sprite.color =
                    color.with_alpha(blink * 0.12 + if used { success_pulse * 0.1 } else { 0.0 })
            }
            Part::Trace(index) => {
                sprite.custom_size = Some(Vec2::new(
                    1.1,
                    if used {
                        if index == 4 { 5.0 } else { 1.2 }
                    } else {
                        2.5 + (t * 2.0 + index as f32 * 1.4).sin() * 1.6
                    },
                ));
                sprite.color = color.with_alpha(if used { 0.65 + blink * 0.3 } else { 0.75 });
            }
        }
    }
    let done = resolved(&map, &state);
    for (light, mut sprite) in &mut lights {
        let base = if done {
            Color::srgb(0.38, 0.78, 0.64).with_alpha(light.base.alpha())
        } else {
            light.base
        };
        sprite.color = base.with_alpha((base.alpha() * (1.0 + success_pulse * 1.2)).min(1.0));
    }
}
pub fn update_ventilation(
    time: Res<Time>,
    screen: Res<State<crate::AppScreen>>,
    map: Res<CurrentLevelMap>,
    state: Res<LocalLevelState>,
    audio: Res<crate::audio_settings::AudioSettings>,
    mut vents: Query<&mut AudioSink, With<Ventilation>>,
) {
    let target = if *screen.get() == crate::AppScreen::InGame && resolved(&map, &state) {
        audio.sfx * 0.11
    } else {
        0.0
    };
    for mut sink in &mut vents {
        let value = sink.volume().to_linear();
        sink.set_volume(bevy::audio::Volume::Linear(
            value + (target - value) * (1.0 - (-time.delta_secs() * 1.5).exp()),
        ));
    }
}
