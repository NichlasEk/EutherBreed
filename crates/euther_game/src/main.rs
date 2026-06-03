mod components;
mod editor;
mod geometry;
mod resources;
mod setup;
mod systems;

use bevy::app::AppExit;
use bevy::prelude::*;
use components::{
    GameOverAction, GameOverEntity, LevelEntity, MainMenuAction, MainMenuEntity, PauseMenuAction,
    PauseMenuEntity,
};
use resources::{
    ApothecaryVitals, CampaignRuntime, CampaignSignal, ContaminantSpawnTimer, CurrentLevelMap,
    GameNotice, LevelRuntime, LocalLevelState, PendingTransition, PersistentLevelStates, RunLives,
    SaveSlot,
};
use setup::{apothecary_spawn_position, setup};
use systems::{
    aim_apothecary, animate_apothecary_walk, apply_save_to_runtime, collect_pickups,
    fire_syringe_round, interact_with_terminals, move_apothecary, move_contaminants,
    move_projectiles, quick_load_on_key, quick_save_on_key, render_map_overlay_on_shift,
    report_exit_overlap, resolve_contaminant_contact, resolve_projectile_hits,
    restart_current_level_on_death, spawn_contaminants, sync_camera_to_level,
    toggle_fullscreen_on_f11, trigger_transition_zones, unlock_doors, update_campaign_progress,
    update_contaminant_hit_flash, update_door_openings, update_effect_lifetimes,
    update_notice_text, update_objective_text, update_pending_transition, update_prompt_text,
    update_section_text, update_status_text,
};

const CONTAMINANT_SPAWN_SECONDS: f32 = 1.7;
const DEFAULT_EDITOR_LEVEL: &str = "research_spine";

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppScreen {
    #[default]
    MainMenu,
    InGame,
    Paused,
    GameOver,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum PauseTab {
    #[default]
    Status,
    Inventory,
    Map,
}

#[derive(Resource, Default)]
struct PauseMenuState {
    tab: PauseTab,
}

#[derive(Component)]
struct PauseMenuText;

fn main() {
    if let Some(level_id) = argument_value("--editor") {
        editor::run_editor(level_id);
        return;
    }

    if let Some(level_id) = argument_value("--editor-smoke") {
        editor::run_editor_smoke(level_id);
        return;
    }

    if std::env::args().any(|arg| arg == "--menu-smoke") {
        run_menu_smoke();
        return;
    }

    if std::env::args().any(|arg| arg == "--headless-smoke") {
        run_headless_smoke();
        return;
    }

    if std::env::args().any(|arg| arg == "--validate-content") {
        validate_content();
        return;
    }

    if std::env::args().any(|arg| arg == "--save-smoke") {
        run_save_smoke();
        return;
    }

    if let Some(path) = argument_value("--save-file-smoke") {
        run_save_file_smoke(path);
        return;
    }

    if let Some(path) = argument_value("--load-file-smoke") {
        run_load_file_smoke(path);
        return;
    }

    if let Some(path) = argument_value("--runtime-save-smoke") {
        run_runtime_save_smoke(path);
        return;
    }

    if let Some(path) = argument_value("--autosave-smoke") {
        run_autosave_smoke(path);
        return;
    }

    if std::env::args().any(|arg| arg == "--entry-smoke") {
        run_entry_smoke();
        return;
    }

    if std::env::args().any(|arg| arg == "--notice-smoke") {
        run_notice_smoke();
        return;
    }

    run_game();
}

fn run_game() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.015, 0.018, 0.025)))
        .insert_resource(initial_vitals())
        .insert_resource(RunLives::default())
        .insert_resource(initial_contaminant_timer())
        .insert_resource(LocalLevelState::default())
        .insert_resource(PersistentLevelStates::default())
        .insert_resource(CampaignSignal::default())
        .insert_resource(PendingTransition::default())
        .insert_resource(initial_campaign_runtime())
        .insert_resource(initial_level_runtime())
        .insert_resource(CurrentLevelMap::default())
        .insert_resource(initial_save_slot())
        .insert_resource(GameNotice::default())
        .insert_resource(PauseMenuState::default())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "EutherBreed Prototype".to_string(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "../../assets".to_string(),
                    ..default()
                }),
        )
        .init_state::<AppScreen>()
        .add_systems(Startup, spawn_menu_camera)
        .add_systems(OnEnter(AppScreen::MainMenu), spawn_main_menu)
        .add_systems(
            Update,
            main_menu_input.run_if(in_state(AppScreen::MainMenu)),
        )
        .add_systems(OnExit(AppScreen::MainMenu), despawn_main_menu)
        .add_systems(OnEnter(AppScreen::InGame), setup)
        .add_systems(OnEnter(AppScreen::Paused), spawn_pause_menu)
        .add_systems(
            Update,
            (pause_menu_input, update_pause_menu_text).run_if(in_state(AppScreen::Paused)),
        )
        .add_systems(OnExit(AppScreen::Paused), despawn_pause_menu)
        .add_systems(OnEnter(AppScreen::GameOver), spawn_game_over_menu)
        .add_systems(
            Update,
            game_over_menu_input.run_if(in_state(AppScreen::GameOver)),
        )
        .add_systems(OnExit(AppScreen::GameOver), despawn_game_over_menu)
        .add_systems(
            Update,
            (
                (move_apothecary, aim_apothecary, animate_apothecary_walk).chain(),
                fire_syringe_round,
                move_projectiles,
                spawn_contaminants,
                move_contaminants,
                resolve_projectile_hits,
                update_contaminant_hit_flash,
                update_effect_lifetimes,
                resolve_contaminant_contact,
                collect_pickups,
            )
                .run_if(in_state(AppScreen::InGame)),
        )
        .add_systems(
            Update,
            (
                quick_save_on_key,
                quick_load_on_key,
                unlock_doors,
                update_door_openings,
                interact_with_terminals,
                trigger_transition_zones,
                update_pending_transition,
                report_exit_overlap,
                update_campaign_progress,
                restart_current_level_on_death,
                update_status_text,
                update_section_text,
                update_objective_text,
                update_prompt_text,
                update_notice_text,
                sync_camera_to_level,
                render_map_overlay_on_shift,
                toggle_fullscreen_on_f11,
                open_pause_menu,
            )
                .run_if(in_state(AppScreen::InGame)),
        )
        .run();
}

fn spawn_menu_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainMenuEntity));
}

fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(18),
                ..default()
            },
            BackgroundColor(Color::srgb(0.008, 0.010, 0.014)),
            MainMenuEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("EutherBreed"),
                TextFont {
                    font_size: 54.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 1.0, 0.92)),
                MainMenuEntity,
            ));
            parent.spawn((
                Text::new("prototype command deck"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.42, 0.62, 0.68)),
                MainMenuEntity,
            ));

            spawn_menu_button(parent, "PLAY", MainMenuAction::Play);
            spawn_menu_button(parent, "EDITOR", MainMenuAction::Editor);
            spawn_menu_button(parent, "QUIT", MainMenuAction::Quit);
        });
}

fn spawn_menu_button(parent: &mut ChildSpawnerCommands, label: &str, action: MainMenuAction) {
    parent
        .spawn((
            Button,
            Node {
                width: px(280),
                height: px(48),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.05, 0.055, 0.95)),
            BorderColor::all(Color::srgba(0.20, 0.95, 0.84, 0.52)),
            action,
            MainMenuEntity,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.84, 0.96, 0.90)),
                MainMenuEntity,
            ));
        });
}

fn main_menu_input(
    mut interactions: Query<
        (
            &Interaction,
            &MainMenuAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action, mut background, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                match action {
                    MainMenuAction::Play => next_state.set(AppScreen::InGame),
                    MainMenuAction::Editor => {
                        launch_editor_process(DEFAULT_EDITOR_LEVEL);
                        exit.write(AppExit::Success);
                    }
                    MainMenuAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
                background.0 = Color::srgba(0.10, 0.28, 0.26, 0.98);
                *border = BorderColor::all(Color::srgba(1.0, 0.78, 0.26, 0.88));
            }
            Interaction::Hovered => {
                background.0 = Color::srgba(0.045, 0.12, 0.12, 0.96);
                *border = BorderColor::all(Color::srgba(0.34, 1.0, 0.88, 0.78));
            }
            Interaction::None => {
                background.0 = Color::srgba(0.025, 0.05, 0.055, 0.95);
                *border = BorderColor::all(Color::srgba(0.20, 0.95, 0.84, 0.52));
            }
        }
    }
}

fn despawn_main_menu(mut commands: Commands, menu_entities: Query<Entity, With<MainMenuEntity>>) {
    for entity in &menu_entities {
        commands.entity(entity).despawn();
    }
}

fn launch_editor_process(level_id: &str) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let _ = std::process::Command::new(exe)
        .arg("--editor")
        .arg(level_id)
        .spawn();
}

fn open_pause_menu(
    input: Res<ButtonInput<KeyCode>>,
    mut pause_state: ResMut<PauseMenuState>,
    mut next_state: ResMut<NextState<AppScreen>>,
) {
    let tab = if input.just_pressed(KeyCode::KeyI) {
        Some(PauseTab::Inventory)
    } else if input.just_pressed(KeyCode::KeyM) {
        Some(PauseTab::Map)
    } else if input.just_pressed(KeyCode::Escape) {
        Some(PauseTab::Status)
    } else {
        None
    };

    if let Some(tab) = tab {
        pause_state.tab = tab;
        next_state.set(AppScreen::Paused);
    }
}

fn spawn_pause_menu(
    mut commands: Commands,
    pause_state: Res<PauseMenuState>,
    vitals: Res<ApothecaryVitals>,
    lives: Res<RunLives>,
    level_state: Res<LocalLevelState>,
    campaign_runtime: Res<CampaignRuntime>,
    current_map: Res<CurrentLevelMap>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
            PauseMenuEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: px(680),
                        min_height: px(430),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12),
                        padding: UiRect::all(px(18)),
                        border: UiRect::all(px(2)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.010, 0.018, 0.020, 0.96)),
                    BorderColor::all(Color::srgba(0.20, 0.95, 0.84, 0.62)),
                    PauseMenuEntity,
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("EutherBreed Systems"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.72, 1.0, 0.92)),
                        PauseMenuEntity,
                    ));
                    panel.spawn((
                        Text::new(pause_summary(
                            pause_state.tab,
                            &vitals,
                            &lives,
                            &level_state,
                            &campaign_runtime,
                            current_map.level.as_ref(),
                        )),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.72, 0.86, 0.86)),
                        PauseMenuText,
                        PauseMenuEntity,
                    ));
                    panel
                        .spawn((
                            Node {
                                display: Display::Flex,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: px(10),
                                row_gap: px(10),
                                ..default()
                            },
                            PauseMenuEntity,
                        ))
                        .with_children(|buttons| {
                            spawn_pause_button(buttons, "RESUME", PauseMenuAction::Resume);
                            spawn_pause_button(buttons, "INVENTORY", PauseMenuAction::Inventory);
                            spawn_pause_button(buttons, "MAP", PauseMenuAction::Map);
                            spawn_pause_button(buttons, "MAIN MENU", PauseMenuAction::MainMenu);
                            spawn_pause_button(buttons, "QUIT", PauseMenuAction::Quit);
                        });
                });
        });
}

fn spawn_pause_button(parent: &mut ChildSpawnerCommands, label: &str, action: PauseMenuAction) {
    parent
        .spawn((
            Button,
            Node {
                width: px(126),
                height: px(42),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.05, 0.055, 0.95)),
            BorderColor::all(Color::srgba(0.20, 0.95, 0.84, 0.52)),
            action,
            PauseMenuEntity,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.84, 0.96, 0.90)),
                PauseMenuEntity,
            ));
        });
}

fn pause_menu_input(
    input: Res<ButtonInput<KeyCode>>,
    mut interactions: Query<
        (
            &Interaction,
            &PauseMenuAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut pause_state: ResMut<PauseMenuState>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
    level_entities: Query<Entity, With<LevelEntity>>,
    mut current_map: ResMut<CurrentLevelMap>,
    mut level_runtime: ResMut<LevelRuntime>,
) {
    if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::Enter) {
        next_state.set(AppScreen::InGame);
    }
    if input.just_pressed(KeyCode::KeyI) {
        pause_state.tab = PauseTab::Inventory;
    }
    if input.just_pressed(KeyCode::KeyM) {
        pause_state.tab = PauseTab::Map;
    }
    if input.just_pressed(KeyCode::Tab) {
        pause_state.tab = match pause_state.tab {
            PauseTab::Status => PauseTab::Inventory,
            PauseTab::Inventory => PauseTab::Map,
            PauseTab::Map => PauseTab::Status,
        };
    }

    for (interaction, action, mut background, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                match action {
                    PauseMenuAction::Resume => next_state.set(AppScreen::InGame),
                    PauseMenuAction::Inventory => {
                        pause_state.tab = PauseTab::Inventory;
                    }
                    PauseMenuAction::Map => {
                        pause_state.tab = PauseTab::Map;
                    }
                    PauseMenuAction::MainMenu => {
                        for entity in &level_entities {
                            commands.entity(entity).despawn();
                        }
                        current_map.level = None;
                        level_runtime.loaded_level_id = None;
                        next_state.set(AppScreen::MainMenu);
                    }
                    PauseMenuAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
                background.0 = Color::srgba(0.10, 0.28, 0.26, 0.98);
                *border = BorderColor::all(Color::srgba(1.0, 0.78, 0.26, 0.88));
            }
            Interaction::Hovered => {
                background.0 = Color::srgba(0.045, 0.12, 0.12, 0.96);
                *border = BorderColor::all(Color::srgba(0.34, 1.0, 0.88, 0.78));
            }
            Interaction::None => {
                background.0 = Color::srgba(0.025, 0.05, 0.055, 0.95);
                *border = BorderColor::all(Color::srgba(0.20, 0.95, 0.84, 0.52));
            }
        }
    }
}

fn update_pause_menu_text(
    pause_state: Res<PauseMenuState>,
    vitals: Res<ApothecaryVitals>,
    lives: Res<RunLives>,
    level_state: Res<LocalLevelState>,
    campaign_runtime: Res<CampaignRuntime>,
    current_map: Res<CurrentLevelMap>,
    mut text_query: Query<&mut Text, With<PauseMenuText>>,
) {
    if !pause_state.is_changed()
        && !vitals.is_changed()
        && !lives.is_changed()
        && !level_state.is_changed()
    {
        return;
    }

    let content = pause_summary(
        pause_state.tab,
        &vitals,
        &lives,
        &level_state,
        &campaign_runtime,
        current_map.level.as_ref(),
    );
    for mut text in &mut text_query {
        **text = content.clone();
    }
}

fn despawn_pause_menu(mut commands: Commands, entities: Query<Entity, With<PauseMenuEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn spawn_game_over_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.76)),
            GameOverEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SUIT BREACH FATAL"),
                TextFont {
                    font_size: 38.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.45, 0.24)),
                GameOverEntity,
            ));
            parent.spawn((
                Text::new("run terminated - containment record archived"),
                TextFont {
                    font_size: 17.0,
                    ..default()
                },
                TextColor(Color::srgb(0.58, 0.82, 0.80)),
                GameOverEntity,
            ));
            spawn_game_over_button(parent, "CONTINUE", GameOverAction::Continue);
            spawn_game_over_button(parent, "MAIN MENU", GameOverAction::MainMenu);
            spawn_game_over_button(parent, "QUIT", GameOverAction::Quit);
        });
}

fn spawn_game_over_button(parent: &mut ChildSpawnerCommands, label: &str, action: GameOverAction) {
    parent
        .spawn((
            Button,
            Node {
                width: px(280),
                height: px(46),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.035, 0.040, 0.96)),
            BorderColor::all(Color::srgba(0.95, 0.35, 0.22, 0.56)),
            action,
            GameOverEntity,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.94, 0.88)),
                GameOverEntity,
            ));
        });
}

fn game_over_menu_input(
    mut commands: Commands,
    mut interactions: Query<
        (
            &Interaction,
            &GameOverAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut exit: MessageWriter<AppExit>,
    mut vitals: ResMut<ApothecaryVitals>,
    mut lives: ResMut<RunLives>,
    mut level_state: ResMut<LocalLevelState>,
    mut persistent_level_states: ResMut<PersistentLevelStates>,
    mut campaign_runtime: ResMut<CampaignRuntime>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut current_map: ResMut<CurrentLevelMap>,
    mut campaign_signal: ResMut<CampaignSignal>,
    mut pending_transition: ResMut<PendingTransition>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
    mut notice: ResMut<GameNotice>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    for (interaction, action, mut background, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                match action {
                    GameOverAction::Continue => {
                        reset_run_state(
                            &mut commands,
                            &level_entities,
                            &mut vitals,
                            &mut lives,
                            &mut level_state,
                            &mut persistent_level_states,
                            &mut campaign_runtime,
                            &mut level_runtime,
                            &mut current_map,
                            &mut campaign_signal,
                            &mut pending_transition,
                            &mut contaminant_timer,
                            &mut notice,
                        );
                        next_state.set(AppScreen::InGame);
                    }
                    GameOverAction::MainMenu => {
                        reset_run_state(
                            &mut commands,
                            &level_entities,
                            &mut vitals,
                            &mut lives,
                            &mut level_state,
                            &mut persistent_level_states,
                            &mut campaign_runtime,
                            &mut level_runtime,
                            &mut current_map,
                            &mut campaign_signal,
                            &mut pending_transition,
                            &mut contaminant_timer,
                            &mut notice,
                        );
                        next_state.set(AppScreen::MainMenu);
                    }
                    GameOverAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
                background.0 = Color::srgba(0.22, 0.06, 0.04, 0.98);
                *border = BorderColor::all(Color::srgba(1.0, 0.70, 0.24, 0.90));
            }
            Interaction::Hovered => {
                background.0 = Color::srgba(0.10, 0.04, 0.035, 0.96);
                *border = BorderColor::all(Color::srgba(1.0, 0.44, 0.24, 0.78));
            }
            Interaction::None => {
                background.0 = Color::srgba(0.035, 0.035, 0.040, 0.96);
                *border = BorderColor::all(Color::srgba(0.95, 0.35, 0.22, 0.56));
            }
        }
    }
}

fn reset_run_state(
    commands: &mut Commands,
    level_entities: &Query<Entity, With<LevelEntity>>,
    vitals: &mut ApothecaryVitals,
    lives: &mut RunLives,
    level_state: &mut LocalLevelState,
    persistent_level_states: &mut PersistentLevelStates,
    campaign_runtime: &mut CampaignRuntime,
    level_runtime: &mut LevelRuntime,
    current_map: &mut CurrentLevelMap,
    campaign_signal: &mut CampaignSignal,
    pending_transition: &mut PendingTransition,
    contaminant_timer: &mut ContaminantSpawnTimer,
    notice: &mut GameNotice,
) {
    for entity in level_entities.iter() {
        commands.entity(entity).despawn();
    }

    *vitals = initial_vitals();
    *lives = RunLives::default();
    *level_state = LocalLevelState::default();
    *persistent_level_states = PersistentLevelStates::default();
    *campaign_runtime = initial_campaign_runtime();
    *level_runtime = initial_level_runtime();
    *current_map = CurrentLevelMap::default();
    *campaign_signal = CampaignSignal::default();
    *pending_transition = PendingTransition::default();
    *contaminant_timer = initial_contaminant_timer();
    notice.clear();
}

fn despawn_game_over_menu(mut commands: Commands, entities: Query<Entity, With<GameOverEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn pause_summary(
    tab: PauseTab,
    vitals: &ApothecaryVitals,
    lives: &RunLives,
    level_state: &LocalLevelState,
    campaign_runtime: &CampaignRuntime,
    level: Option<&game_core::LevelDefinition>,
) -> String {
    let level_name = campaign_runtime.progress.current_level();
    let completed_objectives = level
        .map(|level| {
            level
                .objectives
                .iter()
                .filter(|objective| level_state.0.objectives.is_complete(&objective.id))
                .count()
        })
        .unwrap_or(0);
    match tab {
        PauseTab::Status => format!(
            "STATUS\n\nSection: {level_name}\nLives: {}\nHealth: {}\nReagent rounds: {}\nBio samples: {}\nClearances: {}\nObjectives complete: {}\n\nShortcuts: Esc pause/resume, I inventory, M map, Shift quick map overlay.",
            lives.remaining.max(0),
            vitals.0.health,
            vitals.0.ammo,
            vitals.0.bio_samples,
            level_state.0.clearances.len(),
            completed_objectives,
        ),
        PauseTab::Inventory => format!(
            "INVENTORY\n\nReagent rounds: {}\nBio samples: {}\nArea scan: {}\nAccess tokens: {}\n{}\nRecovered item ids: {}\n{}",
            vitals.0.ammo,
            vitals.0.bio_samples,
            if level_state.0.area_scan_acquired {
                "online"
            } else {
                "missing"
            },
            level_state.0.clearances.len(),
            list_set("Keycards", &level_state.0.clearances),
            level_state.0.collected_pickups.len(),
            list_set("Recent pickup ids", &level_state.0.collected_pickups),
        ),
        PauseTab::Map => {
            let Some(level) = level else {
                return "MAP\n\nNo level loaded.".to_string();
            };
            format!(
                "MAP\n\nLevel: {}\nBounds: {:.0} x {:.0}\nRooms/sections: {}\nDoors: {} | unlocked: {}\n{}\nExits: {}\n{}\nTerminals: {}\nObjectives:\n{}\nKnown pickups: {}\n\nHold Shift in-game for tactical overlay. This pause map is the strategic route summary.",
                level.name,
                level.bounds.half_extents.x * 2.0,
                level.bounds.half_extents.y * 2.0,
                level.sections.len(),
                level.doors.len(),
                level_state.0.unlocked_doors.len(),
                door_status_lines(level, level_state),
                level.exits.len(),
                exit_lines(level, level_state),
                level.terminals.len(),
                objective_lines(level, level_state),
                level
                    .pickups
                    .len()
                    .saturating_sub(level_state.0.collected_pickups.len()),
            )
        }
    }
}

fn list_set(label: &str, values: &std::collections::HashSet<String>) -> String {
    if values.is_empty() {
        return format!("{label}: none");
    }

    let mut values = values.iter().cloned().collect::<Vec<_>>();
    values.sort();
    values.truncate(8);
    format!("{label}: {}", values.join(", "))
}

fn objective_lines(level: &game_core::LevelDefinition, level_state: &LocalLevelState) -> String {
    if level.objectives.is_empty() {
        return "  none".to_string();
    }

    level
        .objectives
        .iter()
        .map(|objective| {
            let state = if level_state.0.objectives.is_complete(&objective.id) {
                "done"
            } else if objective.required {
                "required"
            } else {
                "optional"
            };
            format!("  {state}: {}", objective.label)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn door_status_lines(level: &game_core::LevelDefinition, level_state: &LocalLevelState) -> String {
    if level.doors.is_empty() {
        return "Door status: none".to_string();
    }

    let mut lines = level
        .doors
        .iter()
        .take(8)
        .map(|door| {
            let state = if level_state.0.has_unlocked_door(&door.id) || !door.starts_locked {
                "openable"
            } else {
                "locked"
            };
            format!("  {state}: {}", door.id)
        })
        .collect::<Vec<_>>();
    if level.doors.len() > lines.len() {
        lines.push(format!("  ...{} more", level.doors.len() - lines.len()));
    }
    format!("Door status:\n{}", lines.join("\n"))
}

fn exit_lines(level: &game_core::LevelDefinition, level_state: &LocalLevelState) -> String {
    if level.exits.is_empty() {
        return "Exit routes: none".to_string();
    }

    let lines = level
        .exits
        .iter()
        .map(|exit| {
            let ready = exit
                .required_objectives
                .iter()
                .all(|objective| level_state.0.objectives.is_complete(objective));
            let state = if ready { "ready" } else { "blocked" };
            format!("  {state}: {} -> {}", level.name, exit.target)
        })
        .collect::<Vec<_>>();
    format!("Exit routes:\n{}", lines.join("\n"))
}

fn run_headless_smoke() {
    let mut app = App::new();
    app.insert_resource(initial_vitals())
        .insert_resource(RunLives::default())
        .insert_resource(initial_contaminant_timer())
        .insert_resource(LocalLevelState::default())
        .insert_resource(PersistentLevelStates::default())
        .insert_resource(CampaignSignal::default())
        .insert_resource(PendingTransition::default())
        .insert_resource(initial_campaign_runtime())
        .insert_resource(initial_level_runtime())
        .insert_resource(CurrentLevelMap::default())
        .insert_resource(initial_save_slot())
        .insert_resource(GameNotice::default())
        .add_plugins(MinimalPlugins);

    app.update();

    let vitals = app.world().resource::<ApothecaryVitals>();
    println!(
        "headless smoke ok: health={} ammo={} bio_samples={}",
        vitals.0.health, vitals.0.ammo, vitals.0.bio_samples
    );
}

fn run_menu_smoke() {
    let mut app = App::new();
    app.insert_resource(initial_vitals())
        .insert_resource(RunLives::default())
        .insert_resource(initial_contaminant_timer())
        .insert_resource(LocalLevelState::default())
        .insert_resource(PersistentLevelStates::default())
        .insert_resource(CampaignSignal::default())
        .insert_resource(PendingTransition::default())
        .insert_resource(initial_campaign_runtime())
        .insert_resource(initial_level_runtime())
        .insert_resource(CurrentLevelMap::default())
        .insert_resource(initial_save_slot())
        .insert_resource(GameNotice::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<AppScreen>();

    app.update();
    let state = app.world().resource::<State<AppScreen>>();
    assert_eq!(state.get(), &AppScreen::MainMenu);

    println!("menu smoke ok");
    println!("state: {:?}", state.get());
}

fn initial_vitals() -> ApothecaryVitals {
    ApothecaryVitals(game_core::ApothecaryVitals::new(100, 48, 0))
}

fn initial_contaminant_timer() -> ContaminantSpawnTimer {
    ContaminantSpawnTimer(Timer::from_seconds(
        CONTAMINANT_SPAWN_SECONDS,
        TimerMode::Repeating,
    ))
}

fn initial_campaign_runtime() -> CampaignRuntime {
    let definition = game_core::CampaignDefinition::from_ron_file("assets/campaigns/prototype.ron")
        .unwrap_or_else(|error| panic!("failed to load prototype campaign: {error:?}"));
    definition
        .load_and_validate_levels()
        .unwrap_or_else(|error| panic!("invalid prototype campaign content: {error:?}"));
    let progress = game_core::CampaignProgress::start(&definition)
        .unwrap_or_else(|error| panic!("invalid prototype campaign: {error:?}"));

    CampaignRuntime {
        definition,
        progress,
    }
}

fn validate_content() {
    let definition = game_core::CampaignDefinition::from_ron_file("assets/campaigns/prototype.ron")
        .unwrap_or_else(|error| panic!("failed to load prototype campaign: {error:?}"));
    let levels = definition
        .load_and_validate_levels()
        .unwrap_or_else(|error| panic!("invalid prototype campaign content: {error:?}"));

    println!("content validation ok");
    println!("campaign: {}", definition.name);
    println!("start_level: {}", definition.start_level);
    println!("levels: {}", levels.len());

    for level in levels {
        let section_links = level
            .sections
            .iter()
            .map(|section| section.connects.len())
            .sum::<usize>()
            + level
                .doors
                .iter()
                .filter(|door| door.connects.is_some())
                .count();
        let locked_sections = level
            .sections
            .iter()
            .filter(|section| {
                matches!(
                    section.access,
                    game_core::SectionAccessKind::LockedDoor
                        | game_core::SectionAccessKind::Transition
                )
            })
            .count();
        let reachable_sections = level.reachable_section_count();
        println!(
            "level: {} walls={} sections={} reachable_sections={} section_links={} locked_sections={} contaminants={} pickups={} doors={} terminals={} objectives={} exits={} transitions={}",
            level.name,
            level.walls.len(),
            level.sections.len(),
            reachable_sections,
            section_links,
            locked_sections,
            level.contaminants.len(),
            level.pickups.len(),
            level.doors.len(),
            level.terminals.len(),
            level.objectives.len(),
            level.exits.len(),
            level.transitions.len(),
        );
    }
}

fn run_save_smoke() {
    let loaded = save_smoke_roundtrip();

    println!("save smoke ok");
    print_save_summary(&loaded);
}

fn run_save_file_smoke(path: String) {
    let save = sample_save_game();
    save.write_to_file(&path)
        .unwrap_or_else(|error| panic!("failed to write save smoke file {path}: {error:?}"));
    let loaded = game_core::SaveGame::read_from_file(&path)
        .unwrap_or_else(|error| panic!("failed to read save smoke file {path}: {error:?}"));

    println!("save file smoke ok");
    println!("path: {path}");
    print_save_summary(&loaded);
}

fn run_load_file_smoke(path: String) {
    let save = game_core::SaveGame::read_from_file(&path)
        .unwrap_or_else(|error| panic!("failed to read save smoke file {path}: {error:?}"));
    let mut vitals = initial_vitals();
    let mut campaign_runtime = initial_campaign_runtime();
    let mut level_state = LocalLevelState::default();
    let mut persistent_level_states = PersistentLevelStates::default();

    apply_save_to_runtime(
        &save,
        &mut vitals,
        &mut campaign_runtime,
        &mut level_state,
        &mut persistent_level_states,
    )
    .unwrap_or_else(|error| {
        panic!(
            "failed to apply save level {}: {error:?}",
            save.run_state.current_level
        )
    });

    println!("load file smoke ok");
    println!("path: {path}");
    print_runtime_summary(&vitals, &campaign_runtime, &level_state);
}

fn run_runtime_save_smoke(path: String) {
    let vitals = initial_vitals();
    let campaign_runtime = initial_campaign_runtime();
    let mut level_state = LocalLevelState::default();
    let mut persistent_level_states = PersistentLevelStates::default();
    level_state.0.grant_clearance("quarantine_green");
    level_state
        .0
        .complete_objective("analyze_contaminant_sample");
    level_state.0.collect_pickup("ward_rounds_a");
    level_state.0.unlock_door("ward_quarantine_green_door");
    level_state.0.activate_terminal("ward_lab_analyzer");
    level_state.0.kill_contaminant("ward_contaminant_alpha");
    let mut lab_state = game_core::LevelState::default();
    lab_state.grant_clearance("lab_blue");
    persistent_level_states
        .0
        .insert("lab_access_corridor".to_string(), lab_state);

    let save = systems::save::build_runtime_save(
        &vitals,
        &campaign_runtime,
        &level_state,
        &persistent_level_states,
        Vec2::new(42.0, -64.0),
    );
    systems::save::write_runtime_save(&path, &save).unwrap_or_else(|error| {
        panic!("failed to write runtime save smoke file {path}: {error:?}")
    });

    let loaded = game_core::SaveGame::read_from_file(&path)
        .unwrap_or_else(|error| panic!("failed to read runtime save smoke file {path}: {error:?}"));
    let mut loaded_vitals = initial_vitals();
    let mut loaded_campaign_runtime = initial_campaign_runtime();
    let mut loaded_level_state = LocalLevelState::default();
    let mut loaded_persistent_level_states = PersistentLevelStates::default();

    apply_save_to_runtime(
        &loaded,
        &mut loaded_vitals,
        &mut loaded_campaign_runtime,
        &mut loaded_level_state,
        &mut loaded_persistent_level_states,
    )
    .unwrap_or_else(|error| {
        panic!(
            "failed to apply runtime save level {}: {error:?}",
            loaded.run_state.current_level
        )
    });

    println!("runtime save smoke ok");
    println!("path: {path}");
    println!(
        "persistent_level_states: {}",
        loaded_persistent_level_states.0.len()
    );
    println!(
        "position: {},{}",
        loaded.run_state.position.x, loaded.run_state.position.y
    );
    print_runtime_summary(
        &loaded_vitals,
        &loaded_campaign_runtime,
        &loaded_level_state,
    );
}

fn run_autosave_smoke(path: String) {
    let vitals = initial_vitals();
    let mut campaign_runtime = initial_campaign_runtime();
    let mut level_state = LocalLevelState::default();
    let mut persistent_level_states = PersistentLevelStates::default();

    level_state.0.grant_clearance("quarantine_green");
    level_state
        .0
        .complete_objective("analyze_contaminant_sample");
    level_state.0.collect_pickup("ward_rounds_a");

    let previous_level = campaign_runtime.progress.current_level().to_string();
    campaign_runtime
        .progress
        .travel_to(&campaign_runtime.definition, "lab_access_corridor")
        .unwrap_or_else(|error| panic!("failed to travel during autosave smoke: {error:?}"));
    persistent_level_states
        .0
        .insert(previous_level, level_state.0);
    let next_level_state = persistent_level_states
        .0
        .get(campaign_runtime.progress.current_level())
        .cloned()
        .unwrap_or_default();
    let level_state = LocalLevelState(next_level_state);

    let save = systems::save::build_runtime_save(
        &vitals,
        &campaign_runtime,
        &level_state,
        &persistent_level_states,
        apothecary_spawn_position(
            &setup::load_level_from_campaign(
                &campaign_runtime,
                campaign_runtime.progress.current_level(),
            ),
            Some("from_quarantine_ward"),
        ),
    );
    systems::save::write_runtime_save(&path, &save)
        .unwrap_or_else(|error| panic!("failed to write autosave smoke file {path}: {error:?}"));

    let loaded = game_core::SaveGame::read_from_file(&path)
        .unwrap_or_else(|error| panic!("failed to read autosave smoke file {path}: {error:?}"));

    println!("autosave smoke ok");
    println!("path: {path}");
    println!("current_level: {}", loaded.run_state.current_level);
    println!(
        "position: {},{}",
        loaded.run_state.position.x, loaded.run_state.position.y
    );
    println!("level_states: {}", loaded.level_states.len());
    println!(
        "previous_pickups: {}",
        loaded
            .level_state("prototype_quarantine_ward")
            .collected_pickups
            .len()
    );
}

fn run_entry_smoke() {
    let campaign_runtime = initial_campaign_runtime();
    let ward = setup::load_level_from_campaign(&campaign_runtime, "prototype_quarantine_ward");
    let lab = setup::load_level_from_campaign(&campaign_runtime, "lab_access_corridor");
    let triage = setup::load_level_from_campaign(&campaign_runtime, "triage_vault");
    let research = setup::load_level_from_campaign(&campaign_runtime, "research_spine");
    let lab_entry = apothecary_spawn_position(&lab, Some("from_quarantine_ward"));
    let ward_entry = apothecary_spawn_position(&ward, Some("from_lab_access_corridor"));
    let triage_entry = apothecary_spawn_position(&triage, Some("from_lab_access_corridor"));
    let lab_from_triage = apothecary_spawn_position(&lab, Some("from_triage_vault"));
    let research_from_lab = apothecary_spawn_position(&research, Some("from_lab_access_corridor"));
    let research_from_triage = apothecary_spawn_position(&research, Some("from_triage_vault"));

    assert_eq!(lab_entry, Vec2::new(-390.0, 0.0));
    assert_eq!(ward_entry, Vec2::new(390.0, 0.0));
    assert_eq!(triage_entry, Vec2::new(-390.0, -168.0));
    assert_eq!(lab_from_triage, Vec2::new(390.0, -168.0));
    assert_eq!(research_from_lab, Vec2::new(-650.0, -285.0));
    assert_eq!(research_from_triage, Vec2::new(650.0, -285.0));

    println!("entry smoke ok");
    println!("lab_from_ward: {},{}", lab_entry.x, lab_entry.y);
    println!("ward_from_lab: {},{}", ward_entry.x, ward_entry.y);
    println!("triage_from_lab: {},{}", triage_entry.x, triage_entry.y);
    println!(
        "lab_from_triage: {},{}",
        lab_from_triage.x, lab_from_triage.y
    );
    println!(
        "research_from_lab: {},{}",
        research_from_lab.x, research_from_lab.y
    );
    println!(
        "research_from_triage: {},{}",
        research_from_triage.x, research_from_triage.y
    );
}

fn run_notice_smoke() {
    let mut notice = GameNotice::default();

    notice.show("Saved", 1.5);
    assert!(notice.is_visible());
    assert_eq!(notice.text, "Saved");

    notice.clear();
    assert!(!notice.is_visible());

    println!("notice smoke ok");
    println!("visible: {}", notice.is_visible());
}

fn save_smoke_roundtrip() -> game_core::SaveGame {
    let save = sample_save_game();
    let content = save
        .to_ron_string()
        .unwrap_or_else(|error| panic!("failed to serialize save smoke: {error:?}"));

    game_core::SaveGame::from_ron_str(&content)
        .unwrap_or_else(|error| panic!("failed to deserialize save smoke: {error:?}"))
}

fn sample_save_game() -> game_core::SaveGame {
    let mut level_state = game_core::LevelState::default();
    level_state.grant_clearance("quarantine_green");
    level_state.complete_objective("analyze_contaminant_sample");

    game_core::SaveGame::new(
        game_core::RunState::new_at(
            game_core::ApothecaryVitals::new(100, 48, 0),
            "prototype_quarantine_ward",
            Vec2::new(-300.0, -170.0),
        ),
        level_state,
    )
}

fn print_save_summary(save: &game_core::SaveGame) {
    println!("version: {}", save.version);
    println!("current_level: {}", save.run_state.current_level);
    println!(
        "position: {},{}",
        save.run_state.position.x, save.run_state.position.y
    );
    println!("level_states: {}", save.level_states.len());
    println!("health: {}", save.run_state.vitals.health);
    println!("ammo: {}", save.run_state.vitals.ammo);
    println!("bio_samples: {}", save.run_state.vitals.bio_samples);
}

fn print_runtime_summary(
    vitals: &ApothecaryVitals,
    campaign_runtime: &CampaignRuntime,
    level_state: &LocalLevelState,
) {
    println!(
        "current_level: {}",
        campaign_runtime.progress.current_level()
    );
    println!("health: {}", vitals.0.health);
    println!("ammo: {}", vitals.0.ammo);
    println!("bio_samples: {}", vitals.0.bio_samples);
    println!("clearances: {}", level_state.0.clearances.len());
    println!(
        "collected_pickups: {}",
        level_state.0.collected_pickups.len()
    );
    println!("unlocked_doors: {}", level_state.0.unlocked_doors.len());
    println!(
        "activated_terminals: {}",
        level_state.0.activated_terminals.len()
    );
    println!(
        "killed_contaminants: {}",
        level_state.0.killed_contaminants.len()
    );
    println!(
        "objective_ready: {}",
        level_state
            .0
            .objectives
            .is_complete("analyze_contaminant_sample")
    );
}

fn argument_value(flag: &str) -> Option<String> {
    let mut args = std::env::args();

    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
    }

    None
}

fn initial_level_runtime() -> LevelRuntime {
    LevelRuntime {
        loaded_level_id: None,
        pending_entry_id: None,
        camera_center: Vec2::ZERO,
        camera_size: Vec2::new(900.0, 520.0),
        dynamic_spawn_points: Vec::new(),
        dynamic_spawn_cursor: 0,
        dynamic_spawn_interval_seconds: CONTAMINANT_SPAWN_SECONDS,
        available_exits: Vec::new(),
    }
}

fn initial_save_slot() -> SaveSlot {
    SaveSlot {
        path: systems::save::default_save_path(),
    }
}
