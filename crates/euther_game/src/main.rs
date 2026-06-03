mod components;
mod editor;
mod geometry;
mod resources;
mod setup;
mod systems;

use bevy::prelude::*;
use resources::{
    ApothecaryVitals, CampaignRuntime, CampaignSignal, ContaminantSpawnTimer, CurrentLevelMap,
    GameNotice, LevelRuntime, LocalLevelState, PendingTransition, PersistentLevelStates, SaveSlot,
};
use setup::{apothecary_spawn_position, setup};
use systems::{
    aim_apothecary, animate_apothecary_walk, apply_save_to_runtime, collect_pickups,
    fire_syringe_round, interact_with_terminals, move_apothecary, move_contaminants,
    move_projectiles, quick_load_on_key, quick_save_on_key, quit_on_escape,
    render_map_overlay_on_shift, report_exit_overlap, resolve_contaminant_contact,
    resolve_projectile_hits, restart_current_level_on_death, spawn_contaminants,
    sync_camera_to_level, toggle_fullscreen_on_f11, trigger_transition_zones, unlock_doors,
    update_campaign_progress, update_contaminant_hit_flash, update_door_openings,
    update_effect_lifetimes, update_notice_text, update_objective_text, update_pending_transition,
    update_prompt_text, update_section_text, update_status_text,
};

const CONTAMINANT_SPAWN_SECONDS: f32 = 1.7;

fn main() {
    if let Some(level_id) = argument_value("--editor") {
        editor::run_editor(level_id);
        return;
    }

    if let Some(level_id) = argument_value("--editor-smoke") {
        editor::run_editor_smoke(level_id);
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
        .add_systems(Startup, setup)
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
            ),
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
                quit_on_escape,
            ),
        )
        .run();
}

fn run_headless_smoke() {
    let mut app = App::new();
    app.insert_resource(initial_vitals())
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
