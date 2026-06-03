use bevy::camera::ScalingMode;
use bevy::prelude::*;
use game_core::level::ContaminantDefinition;
use game_core::{
    AxisAlignedBox, DecorDefinition, DecorKind, DoorKind, LevelDefinition, PickupKind,
    PrototypeEntity, TerminalKind,
};
use ron::ser::PrettyConfig;
use std::fs;
use std::path::PathBuf;

const GRID_SIZE: f32 = 16.0;
const SELECT_RADIUS: f32 = 42.0;

pub fn run_editor(level_id: String) {
    let level_path = level_path(&level_id);
    let level = LevelDefinition::from_ron_file(&level_path).unwrap_or_else(|error| {
        panic!(
            "failed to load editor level {}: {error:?}",
            level_path.display()
        )
    });
    level
        .validate()
        .unwrap_or_else(|error| panic!("invalid editor level {}: {error:?}", level_path.display()));

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.010, 0.012, 0.016)))
        .insert_resource(EditorState::new(level_id, level_path, level))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "EutherBreed Level Editor".to_string(),
                        resolution: (1440, 900).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "../../assets".to_string(),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_editor)
        .add_systems(
            Update,
            (
                editor_camera_controls,
                editor_palette_input,
                editor_select_or_place_on_click,
                editor_edit_input,
                editor_save_input,
                sync_editor_text,
            ),
        )
        .run();
}

pub fn run_editor_smoke(level_id: String) {
    let level_path = level_path(&level_id);
    let level = LevelDefinition::from_ron_file(&level_path).unwrap_or_else(|error| {
        panic!(
            "failed to load editor level {}: {error:?}",
            level_path.display()
        )
    });
    level
        .validate()
        .unwrap_or_else(|error| panic!("invalid editor level {}: {error:?}", level_path.display()));
    let pretty = PrettyConfig::default()
        .depth_limit(6)
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    ron::ser::to_string_pretty(&level, pretty).unwrap_or_else(|error| {
        panic!(
            "failed to serialize editor level {}: {error:?}",
            level_path.display()
        )
    });

    println!("editor smoke ok");
    println!("level: {}", level.name);
    println!("path: {}", level_path.display());
    println!("palette_items: {}", default_palette().len());
    println!(
        "editable: decor={} pickups={} contaminants={} doors={} terminals={}",
        level.decor.len(),
        level.pickups.len(),
        level.contaminants.len(),
        level.doors.len(),
        level.terminals.len()
    );
}

#[derive(Resource)]
struct EditorState {
    level_id: String,
    level_path: PathBuf,
    level: LevelDefinition,
    palette: Vec<PaletteItem>,
    palette_index: usize,
    selected: Option<EditableRef>,
    dirty: bool,
    message: String,
    placement_rotation_degrees: f32,
    last_cursor_world: Vec2,
}

impl EditorState {
    fn new(level_id: String, level_path: PathBuf, level: LevelDefinition) -> Self {
        Self {
            level_id,
            level_path,
            level,
            palette: default_palette(),
            palette_index: 0,
            selected: None,
            dirty: false,
            message: "ready".to_string(),
            placement_rotation_degrees: 0.0,
            last_cursor_world: Vec2::ZERO,
        }
    }

    fn current_palette(&self) -> &PaletteItem {
        &self.palette[self.palette_index]
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PaletteItem {
    Decor(DecorKind),
    Pickup(PickupKind),
    Contaminant,
}

impl PaletteItem {
    fn label(&self) -> String {
        match self {
            PaletteItem::Decor(kind) => format!("decor::{kind:?}"),
            PaletteItem::Pickup(kind) => match kind {
                PickupKind::ReagentRounds(amount) => format!("pickup::rounds({amount})"),
                PickupKind::MedGel(amount) => format!("pickup::medgel({amount})"),
                PickupKind::BioSample => "pickup::bio_sample".to_string(),
                PickupKind::SecurityKeycard(clearance) => format!("pickup::keycard({clearance})"),
                PickupKind::AreaScan => "pickup::area_scan".to_string(),
            },
            PaletteItem::Contaminant => "contaminant".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EditableRef {
    Decor(String),
    Pickup(String),
    Contaminant(String),
    Door(String),
    Terminal(String),
    Exit(usize),
    Transition(String),
    EntryPoint(String),
    SpawnPoint(usize),
}

impl EditableRef {
    fn label(&self) -> String {
        match self {
            EditableRef::Decor(id) => format!("decor::{id}"),
            EditableRef::Pickup(id) => format!("pickup::{id}"),
            EditableRef::Contaminant(id) => format!("contaminant::{id}"),
            EditableRef::Door(id) => format!("door::{id}"),
            EditableRef::Terminal(id) => format!("terminal::{id}"),
            EditableRef::Exit(index) => format!("exit::{index}"),
            EditableRef::Transition(id) => format!("transition::{id}"),
            EditableRef::EntryPoint(id) => format!("entry::{id}"),
            EditableRef::SpawnPoint(index) => format!("spawn::{index}"),
        }
    }
}

#[derive(Component)]
struct EditorEntity {
    editable: EditableRef,
}

#[derive(Component)]
struct EditorVisual;

#[derive(Component)]
struct EditorSelectionRing;

#[derive(Component)]
struct EditorCursor;

#[derive(Component)]
struct EditorStatusText;

fn setup_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<EditorState>,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 960.0,
                min_height: 600.0,
            },
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
    ));

    let center = state.level.bounds.center;
    commands.spawn((
        Sprite::from_color(
            Color::srgba(0.03, 0.05, 0.055, 0.95),
            state.level.bounds.half_extents * 2.0,
        ),
        Transform::from_xyz(center.x, center.y, -20.0),
        EditorVisual,
    ));

    spawn_grid(&mut commands, state.level.bounds);
    spawn_editor_level(&mut commands, &asset_server, &state.level);
    spawn_editor_overlay(&mut commands);
    state.message = format!("editing {}", state.level_id);
}

fn spawn_grid(commands: &mut Commands, bounds: AxisAlignedBox) {
    let min = bounds.center - bounds.half_extents;
    let max = bounds.center + bounds.half_extents;
    let color_major = Color::srgba(0.20, 0.72, 0.70, 0.12);
    let color_minor = Color::srgba(0.18, 0.32, 0.34, 0.06);
    let mut x = (min.x / 64.0).floor() * 64.0;
    while x <= max.x {
        let major = (x / 128.0).round() == x / 128.0;
        commands.spawn((
            Sprite::from_color(
                if major { color_major } else { color_minor },
                Vec2::new(if major { 2.0 } else { 1.0 }, bounds.half_extents.y * 2.0),
            ),
            Transform::from_xyz(x, bounds.center.y, -19.0),
            EditorVisual,
        ));
        x += 64.0;
    }

    let mut y = (min.y / 64.0).floor() * 64.0;
    while y <= max.y {
        let major = (y / 128.0).round() == y / 128.0;
        commands.spawn((
            Sprite::from_color(
                if major { color_major } else { color_minor },
                Vec2::new(bounds.half_extents.x * 2.0, if major { 2.0 } else { 1.0 }),
            ),
            Transform::from_xyz(bounds.center.x, y, -19.0),
            EditorVisual,
        ));
        y += 64.0;
    }
}

fn spawn_editor_level(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level: &LevelDefinition,
) {
    for wall in &level.walls {
        spawn_box(
            commands,
            wall.center,
            wall.half_extents * 2.0,
            Color::srgba(0.10, 0.14, 0.16, 0.96),
            -6.0,
        );
    }

    for section in &level.sections {
        spawn_box(
            commands,
            section.bounds.center,
            section.bounds.half_extents * 2.0,
            section_color(section.kind),
            -18.0,
        );
    }

    for decor in &level.decor {
        spawn_decor(commands, asset_server, decor);
    }

    for pickup in &level.pickups {
        spawn_pickup(commands, asset_server, pickup);
    }

    for contaminant in &level.contaminants {
        spawn_contaminant(commands, asset_server, contaminant);
    }

    for door in &level.doors {
        let (path, color) = door_visual(door.kind);
        let mut sprite = Sprite::from_image(asset_server.load(path));
        sprite.custom_size = Some(door.half_extents * 2.0);
        sprite.color = color;
        commands.spawn((
            sprite,
            Transform::from_xyz(door.position.x, door.position.y, 2.0),
            EditorEntity {
                editable: EditableRef::Door(door.id.clone()),
            },
            EditorVisual,
        ));
    }

    for terminal in &level.terminals {
        let mut sprite = Sprite::from_image(asset_server.load(terminal_path(&terminal.kind)));
        sprite.custom_size = Some(Vec2::new(62.0, 58.0));
        sprite.color = terminal_color(&terminal.kind);
        commands.spawn((
            sprite,
            Transform::from_xyz(terminal.position.x, terminal.position.y, 5.0),
            EditorEntity {
                editable: EditableRef::Terminal(terminal.id.clone()),
            },
            EditorVisual,
        ));
    }

    for (index, exit) in level.exits.iter().enumerate() {
        spawn_marker(
            commands,
            asset_server,
            exit.position,
            exit.half_extents * 2.0,
            Color::srgba(0.28, 1.0, 0.86, 0.78),
            EditableRef::Exit(index),
        );
    }

    for transition in &level.transitions {
        spawn_marker(
            commands,
            asset_server,
            transition.position,
            transition.half_extents * 2.0,
            Color::srgba(0.18, 0.92, 1.0, 0.78),
            EditableRef::Transition(transition.id.clone()),
        );
    }

    for entry in &level.entry_points {
        spawn_box_entity(
            commands,
            entry.position,
            Vec2::splat(28.0),
            Color::srgba(0.95, 0.76, 0.24, 0.78),
            7.0,
            EditableRef::EntryPoint(entry.id.clone()),
        );
    }

    for (index, spawn_point) in level.spawn_points.iter().enumerate() {
        spawn_box_entity(
            commands,
            *spawn_point,
            Vec2::splat(24.0),
            Color::srgba(1.0, 0.20, 0.36, 0.64),
            7.0,
            EditableRef::SpawnPoint(index),
        );
    }
}

fn spawn_editor_overlay(commands: &mut Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgba(0.12, 1.0, 0.88, 0.38), Vec2::splat(18.0)),
        Transform::from_xyz(0.0, 0.0, 30.0),
        EditorCursor,
        EditorVisual,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgba(1.0, 0.78, 0.20, 0.48), Vec2::splat(78.0)),
        Transform::from_xyz(0.0, 0.0, 29.0),
        Visibility::Hidden,
        EditorSelectionRing,
        EditorVisual,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(8),
            top: px(8),
            width: px(620),
            padding: UiRect::all(px(8)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.012, 0.016, 0.018, 0.88)),
        BorderColor::all(Color::srgba(0.18, 0.95, 0.84, 0.42)),
        Text::new("editor loading"),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgb(0.76, 0.96, 0.90)),
        EditorStatusText,
    ));
}

fn editor_camera_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (mut transform, mut projection) = camera_query.into_inner();
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS)
        && !keys.pressed(KeyCode::ControlLeft)
        && !keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::ArrowDown)
    {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        let speed = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            720.0
        } else {
            360.0
        };
        transform.translation += (direction.normalize() * speed * time.delta_secs()).extend(0.0);
    }

    let Projection::Orthographic(orthographic) = &mut *projection else {
        return;
    };
    if keys.just_pressed(KeyCode::Equal) {
        orthographic.scale = (orthographic.scale * 0.85).max(0.35);
    }
    if keys.just_pressed(KeyCode::Minus) {
        orthographic.scale = (orthographic.scale * 1.18).min(4.0);
    }
}

fn editor_palette_input(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<EditorState>) {
    if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::KeyE) {
        state.palette_index = (state.palette_index + 1) % state.palette.len();
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        state.palette_index = if state.palette_index == 0 {
            state.palette.len() - 1
        } else {
            state.palette_index - 1
        };
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.placement_rotation_degrees = (state.placement_rotation_degrees + 15.0) % 360.0;
    }
}

fn editor_select_or_place_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_entities: Query<(&Transform, &EditorEntity)>,
    mut cursor_query: Single<&mut Transform, (With<EditorCursor>, Without<EditorEntity>)>,
) {
    let (camera, camera_transform) = camera_query.into_inner();
    let Some(cursor_world) = cursor_world_position(&windows, camera, camera_transform) else {
        return;
    };
    let snapped = snap(cursor_world);
    state.last_cursor_world = snapped;
    cursor_query.translation.x = snapped.x;
    cursor_query.translation.y = snapped.y;

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(editable) = nearest_editable(snapped, &editor_entities) {
        state.selected = Some(editable);
        return;
    }

    place_palette_item(&mut commands, &asset_server, &mut state, snapped);
}

fn editor_edit_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<EditorState>,
    mut editor_entities: Query<(Entity, &mut Transform, &EditorEntity)>,
    selection_query: Single<
        (&mut Transform, &mut Visibility),
        (With<EditorSelectionRing>, Without<EditorEntity>),
    >,
) {
    if keys.just_pressed(KeyCode::Space) {
        let position = state.last_cursor_world;
        place_palette_item(&mut commands, &asset_server, &mut state, position);
    }

    if keys.just_pressed(KeyCode::KeyM) {
        let Some(selected) = state.selected.clone() else {
            state.message = "nothing selected".to_string();
            return;
        };
        let position = state.last_cursor_world;
        move_selected_data(&mut state.level, &selected, position);
        for (_, mut transform, editor_entity) in &mut editor_entities {
            if editor_entity.editable == selected {
                transform.translation.x = position.x;
                transform.translation.y = position.y;
            }
        }
        state.dirty = true;
        state.message = format!("moved {}", selected.label());
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        let Some(selected) = state.selected.clone() else {
            return;
        };
        if remove_selected_data(&mut state.level, &selected) {
            for (entity, _, editor_entity) in &mut editor_entities {
                if editor_entity.editable == selected {
                    commands.entity(entity).despawn();
                }
            }
            state.selected = None;
            state.dirty = true;
            state.message = format!("deleted {}", selected.label());
        } else {
            state.message = format!("cannot delete {}", selected.label());
        }
    }

    if keys.just_pressed(KeyCode::KeyR) {
        if let Some(EditableRef::Decor(id)) = state.selected.clone() {
            let mut message = None;
            if let Some(decor) = state.level.decor.iter_mut().find(|decor| decor.id == id) {
                decor.rotation_degrees = (decor.rotation_degrees + 15.0) % 360.0;
                message = Some(format!("rotated {}", decor.id));
                for (_, mut transform, editor_entity) in &mut editor_entities {
                    if editor_entity.editable == EditableRef::Decor(id.clone()) {
                        transform.rotation =
                            Quat::from_rotation_z(decor.rotation_degrees.to_radians());
                    }
                }
            }
            if let Some(message) = message {
                state.dirty = true;
                state.message = message;
            }
        }
    }

    let (mut selection_transform, mut visibility) = selection_query.into_inner();
    if let Some(selected) = &state.selected {
        if let Some(position) = editable_position(&state.level, selected) {
            selection_transform.translation.x = position.x;
            selection_transform.translation.y = position.y;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn editor_save_input(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<EditorState>) {
    let save_pressed = keys.just_pressed(KeyCode::KeyS)
        && (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight));
    if !save_pressed {
        return;
    }

    match save_level(&state.level_path, &state.level) {
        Ok(()) => {
            state.dirty = false;
            state.message = format!("saved {}", state.level_path.display());
        }
        Err(message) => {
            state.message = message;
        }
    }
}

fn sync_editor_text(
    state: Res<EditorState>,
    mut text_query: Query<&mut Text, With<EditorStatusText>>,
) {
    if !state.is_changed() {
        return;
    }
    let selected = state
        .selected
        .as_ref()
        .map(EditableRef::label)
        .unwrap_or_else(|| "none".to_string());
    let dirty = if state.dirty { "dirty" } else { "clean" };
    let content = format!(
        "EutherBreed editor | {} | {}\nPalette: {} | rot {:.0} deg\n{}\nSelected: {}\nCursor: {:.0},{:.0}\nControls: mouse select/place, Space place, M move, R rotate, Delete remove, Tab/Q/E palette, Ctrl+S save, +/- zoom, WASD pan\n{}",
        state.level_id,
        dirty,
        state.current_palette().label(),
        state.placement_rotation_degrees,
        palette_window(&state.palette, state.palette_index),
        selected,
        state.last_cursor_world.x,
        state.last_cursor_world.y,
        state.message
    );
    for mut text in &mut text_query {
        **text = content.clone();
    }
}

fn palette_window(palette: &[PaletteItem], index: usize) -> String {
    let mut labels = Vec::new();
    for offset in 0..5 {
        let item_index = (index + offset) % palette.len();
        let prefix = if offset == 0 { ">" } else { " " };
        labels.push(format!("{prefix} {}", palette[item_index].label()));
    }
    labels.join(" | ")
}

fn place_palette_item(
    commands: &mut Commands,
    asset_server: &AssetServer,
    state: &mut EditorState,
    position: Vec2,
) {
    match state.current_palette().clone() {
        PaletteItem::Decor(kind) => {
            let id = unique_id(
                "editor_decor",
                state.level.decor.iter().map(|decor| decor.id.as_str()),
            );
            let decor = DecorDefinition {
                id: id.clone(),
                position,
                kind,
                rotation_degrees: state.placement_rotation_degrees,
                blocking: false,
            };
            spawn_decor(commands, asset_server, &decor);
            state.level.decor.push(decor);
            state.selected = Some(EditableRef::Decor(id));
        }
        PaletteItem::Pickup(kind) => {
            let id = unique_id(
                "editor_pickup",
                state.level.pickups.iter().map(|pickup| pickup.id.as_str()),
            );
            let pickup = PrototypeEntity {
                id: id.clone(),
                position,
                kind,
            };
            spawn_pickup(commands, asset_server, &pickup);
            state.level.pickups.push(pickup);
            state.selected = Some(EditableRef::Pickup(id));
        }
        PaletteItem::Contaminant => {
            let id = unique_id(
                "editor_contaminant",
                state
                    .level
                    .contaminants
                    .iter()
                    .map(|contaminant| contaminant.id.as_str()),
            );
            let contaminant = ContaminantDefinition {
                id: id.clone(),
                position,
            };
            spawn_contaminant(commands, asset_server, &contaminant);
            state.level.contaminants.push(contaminant);
            state.selected = Some(EditableRef::Contaminant(id));
        }
    }
    state.dirty = true;
    state.message = format!("placed {}", state.selected.as_ref().unwrap().label());
}

fn move_selected_data(level: &mut LevelDefinition, selected: &EditableRef, position: Vec2) {
    match selected {
        EditableRef::Decor(id) => {
            if let Some(decor) = level.decor.iter_mut().find(|decor| decor.id == *id) {
                decor.position = position;
            }
        }
        EditableRef::Pickup(id) => {
            if let Some(pickup) = level.pickups.iter_mut().find(|pickup| pickup.id == *id) {
                pickup.position = position;
            }
        }
        EditableRef::Contaminant(id) => {
            if let Some(contaminant) = level
                .contaminants
                .iter_mut()
                .find(|contaminant| contaminant.id == *id)
            {
                contaminant.position = position;
            }
        }
        EditableRef::Door(id) => {
            if let Some(door) = level.doors.iter_mut().find(|door| door.id == *id) {
                door.position = position;
            }
        }
        EditableRef::Terminal(id) => {
            if let Some(terminal) = level
                .terminals
                .iter_mut()
                .find(|terminal| terminal.id == *id)
            {
                terminal.position = position;
            }
        }
        EditableRef::Exit(index) => {
            if let Some(exit) = level.exits.get_mut(*index) {
                exit.position = position;
            }
        }
        EditableRef::Transition(id) => {
            if let Some(transition) = level
                .transitions
                .iter_mut()
                .find(|transition| transition.id == *id)
            {
                transition.position = position;
            }
        }
        EditableRef::EntryPoint(id) => {
            if let Some(entry) = level.entry_points.iter_mut().find(|entry| entry.id == *id) {
                entry.position = position;
            }
        }
        EditableRef::SpawnPoint(index) => {
            if let Some(spawn) = level.spawn_points.get_mut(*index) {
                *spawn = position;
            }
        }
    }
}

fn remove_selected_data(level: &mut LevelDefinition, selected: &EditableRef) -> bool {
    match selected {
        EditableRef::Decor(id) => remove_by_id(&mut level.decor, id, |decor| &decor.id),
        EditableRef::Pickup(id) => remove_by_id(&mut level.pickups, id, |pickup| &pickup.id),
        EditableRef::Contaminant(id) => {
            remove_by_id(&mut level.contaminants, id, |contaminant| &contaminant.id)
        }
        _ => false,
    }
}

fn editable_position(level: &LevelDefinition, selected: &EditableRef) -> Option<Vec2> {
    match selected {
        EditableRef::Decor(id) => level
            .decor
            .iter()
            .find(|decor| decor.id == *id)
            .map(|decor| decor.position),
        EditableRef::Pickup(id) => level
            .pickups
            .iter()
            .find(|pickup| pickup.id == *id)
            .map(|pickup| pickup.position),
        EditableRef::Contaminant(id) => level
            .contaminants
            .iter()
            .find(|contaminant| contaminant.id == *id)
            .map(|contaminant| contaminant.position),
        EditableRef::Door(id) => level
            .doors
            .iter()
            .find(|door| door.id == *id)
            .map(|door| door.position),
        EditableRef::Terminal(id) => level
            .terminals
            .iter()
            .find(|terminal| terminal.id == *id)
            .map(|terminal| terminal.position),
        EditableRef::Exit(index) => level.exits.get(*index).map(|exit| exit.position),
        EditableRef::Transition(id) => level
            .transitions
            .iter()
            .find(|transition| transition.id == *id)
            .map(|transition| transition.position),
        EditableRef::EntryPoint(id) => level
            .entry_points
            .iter()
            .find(|entry| entry.id == *id)
            .map(|entry| entry.position),
        EditableRef::SpawnPoint(index) => level.spawn_points.get(*index).copied(),
    }
}

fn save_level(path: &PathBuf, level: &LevelDefinition) -> Result<(), String> {
    level
        .validate()
        .map_err(|error| format!("validation failed before save: {error:?}"))?;
    let pretty = PrettyConfig::default()
        .depth_limit(6)
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let content = ron::ser::to_string_pretty(level, pretty)
        .map_err(|error| format!("failed to serialize level: {error:?}"))?;
    fs::write(path, content)
        .map_err(|error| format!("failed to write {}: {error:?}", path.display()))
}

fn cursor_world_position(
    windows: &Query<&Window>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

fn nearest_editable(
    position: Vec2,
    editor_entities: &Query<(&Transform, &EditorEntity)>,
) -> Option<EditableRef> {
    editor_entities
        .iter()
        .filter_map(|(transform, entity)| {
            let distance = transform.translation.xy().distance(position);
            (distance <= SELECT_RADIUS).then_some((distance, entity.editable.clone()))
        })
        .min_by(|(left, _), (right, _)| left.total_cmp(right))
        .map(|(_, editable)| editable)
}

fn spawn_decor(commands: &mut Commands, asset_server: &AssetServer, decor: &DecorDefinition) {
    let (path, size, z) = decor_visual(decor.kind);
    let mut sprite = Sprite::from_image(asset_server.load(path));
    sprite.custom_size = Some(size);
    let mut transform = Transform::from_xyz(decor.position.x, decor.position.y, z);
    transform.rotation = Quat::from_rotation_z(decor.rotation_degrees.to_radians());
    commands.spawn((
        sprite,
        transform,
        EditorEntity {
            editable: EditableRef::Decor(decor.id.clone()),
        },
        EditorVisual,
    ));
}

fn spawn_pickup(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pickup: &PrototypeEntity<PickupKind>,
) {
    let (path, size) = pickup_visual(&pickup.kind);
    let mut sprite = Sprite::from_image(asset_server.load(path));
    sprite.custom_size = Some(size);
    commands.spawn((
        sprite,
        Transform::from_xyz(pickup.position.x, pickup.position.y, 6.0),
        EditorEntity {
            editable: EditableRef::Pickup(pickup.id.clone()),
        },
        EditorVisual,
    ));
}

fn spawn_contaminant(
    commands: &mut Commands,
    asset_server: &AssetServer,
    contaminant: &ContaminantDefinition,
) {
    let mut sprite = Sprite::from_image(asset_server.load("sprites/biomech/contaminant.png"));
    sprite.custom_size = Some(Vec2::new(64.0, 50.0));
    commands.spawn((
        sprite,
        Transform::from_xyz(contaminant.position.x, contaminant.position.y, 9.0),
        EditorEntity {
            editable: EditableRef::Contaminant(contaminant.id.clone()),
        },
        EditorVisual,
    ));
}

fn spawn_marker(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    size: Vec2,
    color: Color,
    editable: EditableRef,
) {
    let mut sprite = Sprite::from_image(asset_server.load("sprites/biomech/exit_marker.png"));
    sprite.custom_size = Some(size);
    sprite.color = color;
    commands.spawn((
        sprite,
        Transform::from_xyz(position.x, position.y, 4.0),
        EditorEntity { editable },
        EditorVisual,
    ));
}

fn spawn_box(commands: &mut Commands, center: Vec2, size: Vec2, color: Color, z: f32) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(center.x, center.y, z),
        EditorVisual,
    ));
}

fn spawn_box_entity(
    commands: &mut Commands,
    center: Vec2,
    size: Vec2,
    color: Color,
    z: f32,
    editable: EditableRef,
) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(center.x, center.y, z),
        EditorEntity { editable },
        EditorVisual,
    ));
}

fn default_palette() -> Vec<PaletteItem> {
    vec![
        PaletteItem::Decor(DecorKind::LabTable),
        PaletteItem::Decor(DecorKind::BioTank),
        PaletteItem::Decor(DecorKind::MedBed),
        PaletteItem::Decor(DecorKind::SupplyCrate),
        PaletteItem::Decor(DecorKind::PipeCluster),
        PaletteItem::Decor(DecorKind::FloorGrate),
        PaletteItem::Decor(DecorKind::HazardFloor),
        PaletteItem::Decor(DecorKind::CrackedPanel),
        PaletteItem::Decor(DecorKind::BloodPool),
        PaletteItem::Decor(DecorKind::BloodSmear),
        PaletteItem::Decor(DecorKind::BloodDrops),
        PaletteItem::Decor(DecorKind::AcidScorch),
        PaletteItem::Decor(DecorKind::CorpsePile),
        PaletteItem::Pickup(PickupKind::ReagentRounds(12)),
        PaletteItem::Pickup(PickupKind::MedGel(24)),
        PaletteItem::Pickup(PickupKind::BioSample),
        PaletteItem::Pickup(PickupKind::AreaScan),
        PaletteItem::Pickup(PickupKind::SecurityKeycard("open".to_string())),
        PaletteItem::Contaminant,
    ]
}

fn decor_visual(kind: DecorKind) -> (&'static str, Vec2, f32) {
    match kind {
        DecorKind::BloodDrops => (
            "sprites/biomech/v2_decor_blood_drops.png",
            Vec2::new(54.0, 42.0),
            -1.0,
        ),
        DecorKind::BloodSmear => (
            "sprites/biomech/v2_decor_blood_smear.png",
            Vec2::new(92.0, 44.0),
            -1.0,
        ),
        DecorKind::BloodPool => (
            "sprites/biomech/v2_decor_blood_pool.png",
            Vec2::new(96.0, 72.0),
            -1.0,
        ),
        DecorKind::AcidScorch => (
            "sprites/biomech/v2_decor_acid_scorch.png",
            Vec2::new(90.0, 70.0),
            -1.0,
        ),
        DecorKind::CrackedPanel => (
            "sprites/biomech/v2_decor_cracked_panel.png",
            Vec2::new(84.0, 84.0),
            -1.0,
        ),
        DecorKind::LabTable => (
            "sprites/biomech/v2_decor_lab_table.png",
            Vec2::new(112.0, 64.0),
            1.0,
        ),
        DecorKind::MedBed => (
            "sprites/biomech/v2_decor_med_bed.png",
            Vec2::new(70.0, 118.0),
            1.0,
        ),
        DecorKind::BioTank => (
            "sprites/biomech/v2_decor_bio_tank.png",
            Vec2::new(62.0, 96.0),
            1.0,
        ),
        DecorKind::SupplyCrate => (
            "sprites/biomech/v2_decor_supply_crate_small.png",
            Vec2::new(58.0, 48.0),
            1.0,
        ),
        DecorKind::PipeCluster => (
            "sprites/biomech/v2_decor_pipe_cluster.png",
            Vec2::new(116.0, 42.0),
            1.0,
        ),
        DecorKind::CorpsePile => (
            "sprites/biomech/v2_decor_corpse_pile.png",
            Vec2::new(88.0, 74.0),
            1.0,
        ),
        DecorKind::FloorGrate => (
            "sprites/biomech/v2_decor_floor_grate.png",
            Vec2::new(112.0, 82.0),
            -1.0,
        ),
        DecorKind::HazardFloor => (
            "sprites/biomech/v2_decor_hazard_floor.png",
            Vec2::new(112.0, 38.0),
            -1.0,
        ),
    }
}

fn pickup_visual(kind: &PickupKind) -> (&'static str, Vec2) {
    match kind {
        PickupKind::ReagentRounds(_) => (
            "sprites/biomech/pickup_reagent_rounds.png",
            Vec2::new(22.0, 32.0),
        ),
        PickupKind::MedGel(_) => ("sprites/biomech/pickup_med_gel.png", Vec2::new(22.0, 34.0)),
        PickupKind::BioSample => (
            "sprites/biomech/pickup_bio_sample.png",
            Vec2::new(20.0, 40.0),
        ),
        PickupKind::SecurityKeycard(_) => (
            "sprites/biomech/pickup_security_keycard.png",
            Vec2::new(34.0, 26.0),
        ),
        PickupKind::AreaScan => (
            "sprites/biomech/pickup_area_scan.png",
            Vec2::new(30.0, 26.0),
        ),
    }
}

fn door_visual(kind: DoorKind) -> (&'static str, Color) {
    match kind {
        DoorKind::Bulkhead => (
            "sprites/biomech/v2_door_bulkhead.png",
            Color::srgba(0.78, 1.0, 0.92, 0.90),
        ),
        DoorKind::EnergyBarrier => (
            "sprites/biomech/v2_door_energy_barrier.png",
            Color::srgba(0.72, 0.45, 1.0, 0.90),
        ),
    }
}

fn terminal_path(kind: &TerminalKind) -> &'static str {
    match kind {
        TerminalKind::LabAnalyzer | TerminalKind::ShipLog | TerminalKind::SupplyConsole => {
            "sprites/biomech/v2_terminal_lab_analyzer.png"
        }
    }
}

fn terminal_color(kind: &TerminalKind) -> Color {
    match kind {
        TerminalKind::LabAnalyzer => Color::WHITE,
        TerminalKind::ShipLog => Color::srgba(0.78, 0.88, 1.0, 1.0),
        TerminalKind::SupplyConsole => Color::srgba(0.72, 1.0, 0.88, 1.0),
    }
}

fn section_color(kind: game_core::SectionKind) -> Color {
    match kind {
        game_core::SectionKind::Corridor => Color::srgba(0.02, 0.08, 0.09, 0.10),
        game_core::SectionKind::Lab => Color::srgba(0.04, 0.24, 0.20, 0.16),
        game_core::SectionKind::Triage => Color::srgba(0.28, 0.07, 0.12, 0.14),
        game_core::SectionKind::Supply => Color::srgba(0.30, 0.16, 0.04, 0.16),
        game_core::SectionKind::Lift => Color::srgba(0.04, 0.24, 0.30, 0.16),
        game_core::SectionKind::Containment => Color::srgba(0.28, 0.03, 0.07, 0.16),
    }
}

fn snap(position: Vec2) -> Vec2 {
    Vec2::new(
        (position.x / GRID_SIZE).round() * GRID_SIZE,
        (position.y / GRID_SIZE).round() * GRID_SIZE,
    )
}

fn level_path(level_id: &str) -> PathBuf {
    PathBuf::from(format!("assets/levels/{level_id}.ron"))
}

fn unique_id<'a>(prefix: &str, existing: impl Iterator<Item = &'a str>) -> String {
    let existing = existing.collect::<Vec<_>>();
    for index in 1.. {
        let candidate = format!("{prefix}_{index:03}");
        if !existing.iter().any(|id| **id == candidate) {
            return candidate;
        }
    }
    unreachable!("infinite id generator exhausted")
}

fn remove_by_id<T>(items: &mut Vec<T>, id: &str, id_fn: impl Fn(&T) -> &String) -> bool {
    let Some(index) = items.iter().position(|item| id_fn(item) == id) else {
        return false;
    };
    items.remove(index);
    true
}
