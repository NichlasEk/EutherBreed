use crate::{AppScreen, resources::GameNotice};
use bevy::{audio::Volume, prelude::*, ui::RelativeCursorPosition};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AudioSettings {
    pub music: f32,
    pub sfx: f32,
    pub music_muted: bool,
}
impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music: 0.32,
            sfx: 0.8,
            music_muted: false,
        }
    }
}
impl AudioSettings {
    fn sanitized(mut self) -> Self {
        self.music = finite_volume(self.music, 0.32);
        self.sfx = finite_volume(self.sfx, 0.8);
        self
    }
}
fn finite_volume(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

#[derive(Resource)]
pub struct SettingsStore {
    path: PathBuf,
    saved: AudioSettings,
}
#[derive(Resource)]
pub struct SettingsReturn(pub AppScreen);
impl Default for SettingsReturn {
    fn default() -> Self {
        Self(AppScreen::MainMenu)
    }
}

pub fn install(app: &mut App) {
    let path = crate::argument_value("--visual-smoke")
        .map(|d| PathBuf::from(d).join("audio-settings.ron"))
        .unwrap_or_else(|| PathBuf::from("saves/audio-settings.ron"));
    let settings = load_settings(&path);
    app.insert_resource(SettingsStore {
        path,
        saved: settings.clone(),
    })
    .insert_resource(settings)
    .init_resource::<SettingsReturn>()
    .add_systems(Update, (save_changed_settings, sync_sfx_volume))
    .add_systems(OnEnter(AppScreen::Settings), spawn_settings)
    .add_systems(Update, settings_input.run_if(in_state(AppScreen::Settings)))
    .add_systems(OnExit(AppScreen::Settings), close_settings);
}
fn load_settings(path: &Path) -> AudioSettings {
    match std::fs::read_to_string(path) {
        Ok(raw) => match ron::from_str::<AudioSettings>(&raw) {
            Ok(value) => value.sanitized(),
            Err(error) => {
                warn!("Invalid audio settings: {error}; using defaults");
                AudioSettings::default()
            }
        },
        Err(_) => AudioSettings::default(),
    }
}
fn write_settings(path: &Path, settings: &AudioSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = ron::ser::to_string_pretty(settings, ron::ser::PrettyConfig::default())
        .map_err(std::io::Error::other)?;
    let temporary = path.with_extension("ron.tmp");
    std::fs::write(&temporary, raw)?;
    std::fs::rename(temporary, path)
}
fn save_changed_settings(
    settings: Res<AudioSettings>,
    mut store: ResMut<SettingsStore>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut notice: ResMut<GameNotice>,
) {
    // Save on release, not for every pixel dragged. Keyboard edits save immediately.
    if mouse.pressed(MouseButton::Left) || *settings == store.saved {
        return;
    }
    match write_settings(&store.path, &settings) {
        Ok(()) => store.saved = settings.clone(),
        Err(error) => {
            warn!("Could not save audio settings: {error}");
            notice.show("Audio settings could not be saved", 2.0);
        }
    }
}

#[derive(Component)]
pub struct SfxVoice;
pub fn play_sfx(
    commands: &mut Commands,
    assets: &AssetServer,
    settings: &AudioSettings,
    path: &'static str,
) {
    if settings.sfx <= 0.0 {
        return;
    }
    commands.spawn((
        AudioPlayer::new(assets.load(path)),
        PlaybackSettings {
            volume: Volume::Linear(settings.sfx),
            ..PlaybackSettings::DESPAWN
        },
        SfxVoice,
        crate::components::LevelEntity,
    ));
}
fn sync_sfx_volume(settings: Res<AudioSettings>, mut sinks: Query<&mut AudioSink, With<SfxVoice>>) {
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(settings.sfx));
    }
}

#[derive(Component)]
struct SettingsRoot;
#[derive(Component, Clone, Copy, PartialEq)]
pub(crate) enum Channel {
    Music,
    Sfx,
}
#[derive(Component)]
struct SliderFill(Channel);
#[derive(Component)]
struct SliderLabel(Channel);
#[derive(Component)]
struct BackButton;
#[derive(Resource, Default)]
struct SliderFocus {
    selected: Option<Channel>,
    dragging: Option<Channel>,
}
fn channel_value(settings: &AudioSettings, channel: Channel) -> f32 {
    match channel {
        Channel::Music => {
            if settings.music_muted {
                0.0
            } else {
                settings.music
            }
        }
        Channel::Sfx => settings.sfx,
    }
}
fn set_channel(settings: &mut AudioSettings, channel: Channel, value: f32) {
    let value = (value.clamp(0.0, 1.0) * 100.0).round() / 100.0;
    match channel {
        Channel::Music => {
            settings.music = value;
            settings.music_muted = false;
        }
        Channel::Sfx => settings.sfx = value,
    }
}
fn spawn_settings(mut commands: Commands, settings: Res<AudioSettings>, assets: Res<AssetServer>) {
    let font = assets.load("fonts/NotoSans-Regular.ttf");
    commands.insert_resource(SliderFocus {
        selected: Some(Channel::Music),
        dragging: None,
    });
    commands.spawn((Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }, BackgroundColor(Color::srgba(0.004, 0.01, 0.016, 0.97)), GlobalZIndex(100), SettingsRoot))
    .with_children(|root| {
        root.spawn((Node { width: percent(86), max_width: px(600), padding: UiRect::all(px(28)), flex_direction: FlexDirection::Column, row_gap: px(22), border: UiRect::all(px(1)), ..default() }, BackgroundColor(Color::srgb(0.018,0.035,0.044)), BorderColor::all(Color::srgb(0.20,0.55,0.60))))
        .with_children(|panel| {
            panel.spawn((Text::new("LJUDINSTÄLLNINGAR"), TextFont { font: font.clone(), font_size: 26.0, ..default() }, TextColor(Color::srgb(0.7,0.95,0.93))));
            for channel in [Channel::Music, Channel::Sfx] {
                panel.spawn((Text::new(label(&settings, channel)), TextFont { font: font.clone(), font_size: 20.0, ..default() }, SliderLabel(channel)));
                panel.spawn((Button, RelativeCursorPosition::default(), channel, Node { width: percent(100), height: px(36), border: UiRect::all(px(2)), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.04,0.08,0.10)), BorderColor::all(Color::srgb(0.20,0.55,0.60))))
                .with_children(|track| {
                    track.spawn((Node { width: percent(channel_value(&settings, channel)*100.0), height: percent(100), ..default() }, BackgroundColor(Color::srgb(0.13,0.67,0.72)), SliderFill(channel)));
                });
            }
            panel.spawn((Text::new("Dra reglagen. Tab väljer. Vänster/höger justerar.\nInställningarna sparas automatiskt."), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::srgb(0.55,0.72,0.75))));
            panel.spawn((Button, BackButton, Node { height: px(44), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(px(1)), ..default() }, BackgroundColor(Color::srgb(0.04,0.12,0.14)), BorderColor::all(Color::srgb(0.2,0.55,0.6))))
                .with_children(|button| { button.spawn(Text::new("Tillbaka [Esc]")); });
        });
    });
}
fn label(settings: &AudioSettings, channel: Channel) -> String {
    format!(
        "{}   {}%",
        if channel == Channel::Music {
            "Musik"
        } else {
            "Ljudeffekter"
        },
        (channel_value(settings, channel) * 100.0).round() as i32
    )
}
fn settings_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<AudioSettings>,
    mut focus: ResMut<SliderFocus>,
    mut tracks: Query<(
        &Channel,
        &RelativeCursorPosition,
        &Interaction,
        &mut BorderColor,
    )>,
    mut fills: Query<(&SliderFill, &mut Node)>,
    mut labels: Query<(&SliderLabel, &mut Text)>,
    back: Query<&Interaction, (With<BackButton>, Changed<Interaction>)>,
    target: Res<SettingsReturn>,
    mut next: ResMut<NextState<AppScreen>>,
    assets: Res<AssetServer>,
) {
    if keys.just_pressed(KeyCode::Escape) || back.iter().any(|i| *i == Interaction::Pressed) {
        next.set(target.0);
    }
    if keys.just_pressed(KeyCode::Tab) {
        focus.selected = Some(if focus.selected == Some(Channel::Music) {
            Channel::Sfx
        } else {
            Channel::Music
        });
    }
    for (channel, cursor, interaction, mut border) in &mut tracks {
        if *interaction == Interaction::Pressed && mouse.just_pressed(MouseButton::Left) {
            focus.selected = Some(*channel);
            focus.dragging = Some(*channel);
        }
        if mouse.pressed(MouseButton::Left) && focus.dragging == Some(*channel) {
            if let Some(point) = cursor.normalized {
                set_channel(&mut settings, *channel, point.x + 0.5);
            }
        }
        *border = BorderColor::all(if focus.selected == Some(*channel) {
            Color::srgb(0.6, 0.95, 0.9)
        } else {
            Color::srgb(0.20, 0.55, 0.60)
        });
    }
    let step = i32::from(keys.just_pressed(KeyCode::ArrowRight))
        - i32::from(keys.just_pressed(KeyCode::ArrowLeft));
    if let Some(channel) = focus.selected {
        if step != 0 {
            let value = channel_value(&settings, channel) + step as f32 * 0.05;
            set_channel(&mut settings, channel, value);
        }
    }
    if (mouse.just_released(MouseButton::Left) && focus.dragging == Some(Channel::Sfx))
        || (step != 0 && focus.selected == Some(Channel::Sfx))
    {
        play_sfx(&mut commands, &assets, &settings, "audio/terminal-used.ogg");
    }
    if !mouse.pressed(MouseButton::Left) {
        focus.dragging = None;
    }
    for (fill, mut node) in &mut fills {
        node.width = percent(channel_value(&settings, fill.0) * 100.0);
    }
    for (channel, mut text) in &mut labels {
        text.0 = label(&settings, channel.0);
    }
}
fn close_settings(mut commands: Commands, roots: Query<Entity, With<SettingsRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn volumes_roundtrip_and_sanitize_without_changing_each_other() {
        let path =
            std::env::temp_dir().join(format!("eutherbreed-audio-test-{}.ron", std::process::id()));
        let mut value = AudioSettings::default();
        set_channel(&mut value, Channel::Sfx, 0.17);
        assert_eq!(value.music, 0.32);
        write_settings(&path, &value).unwrap();
        assert_eq!(load_settings(&path), value);
        std::fs::remove_file(path).unwrap();
        let clean = AudioSettings {
            music: f32::NAN,
            sfx: 5.0,
            music_muted: true,
        }
        .sanitized();
        assert_eq!(clean.music, 0.32);
        assert_eq!(clean.sfx, 1.0);
        assert!(clean.music_muted);
    }
}
