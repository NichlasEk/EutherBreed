use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use game_core::{
    AxisAlignedBox, DecorDefinition, DecorKind, DoorDefinition, DoorKind, LevelDefinition,
    PickupKind, TerminalKind, TransitionKind,
};
use std::time::Duration;

use crate::components::{
    Apothecary, ApothecaryAnimation, BioText, Contaminant, ContaminantAnimation, Door, ExitZone,
    HudGaugeKind, HudGaugePip, KeysText, LevelEntity, NoticeText, ObjectiveText, Pickup,
    PromptText, SectionText, Terminal, TransitionZone, Wall,
};
use crate::resources::{
    CampaignRuntime, ContaminantSpawnTimer, CurrentLevelMap, LevelRuntime, LocalLevelState,
};

const FLOOR_VISUAL_BLEED: Vec2 = Vec2::new(960.0, 480.0);
const FLOOR_TILE_PATHS: [&str; 4] = [
    "sprites/biomech/tile_floor_biomech.png",
    "sprites/biomech/tile_floor_biomech_b.png",
    "sprites/biomech/tile_floor_biomech_c.png",
    "sprites/biomech/tile_floor_biomech_d.png",
];
const HUD_RAIL_TOP_PATH: &str = "sprites/ui/hud_rail_top.png";
const HUD_RAIL_BOTTOM_PATH: &str = "sprites/ui/hud_rail_bottom.png";
const HUD_SEGMENT_CYAN_PATH: &str = "sprites/ui/hud_segment_cyan.png";
const HUD_SEGMENT_STEEL_PATH: &str = "sprites/ui/hud_segment_steel.png";

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    campaign_runtime: Res<CampaignRuntime>,
    level_state: Res<LocalLevelState>,
    mut level_runtime: ResMut<LevelRuntime>,
    mut current_level_map: ResMut<CurrentLevelMap>,
    mut contaminant_timer: ResMut<ContaminantSpawnTimer>,
) {
    commands.spawn(Camera2d);

    spawn_hud(&mut commands, &asset_server);

    let current_level_id = campaign_runtime.progress.current_level().to_string();
    let level = load_level_from_campaign(&campaign_runtime, &current_level_id);
    spawn_level(
        &mut commands,
        &asset_server,
        &level,
        &level_state.0,
        None,
        None,
    );
    update_level_runtime(
        &mut level_runtime,
        &mut current_level_map,
        &level,
        &mut contaminant_timer,
    );

    level_runtime.loaded_level_id = Some(current_level_id);
}

pub fn load_level_from_campaign(
    campaign_runtime: &CampaignRuntime,
    level_id: &str,
) -> LevelDefinition {
    let campaign_level = campaign_runtime
        .definition
        .levels
        .iter()
        .find(|level| level.id == level_id)
        .unwrap_or_else(|| panic!("campaign level not found: {level_id}"));

    let level = LevelDefinition::from_ron_file(&campaign_level.path)
        .unwrap_or_else(|error| panic!("failed to load level {}: {error:?}", campaign_level.path));

    level
        .validate()
        .unwrap_or_else(|error| panic!("invalid level {}: {error:?}", campaign_level.path));

    level
}

fn spawn_hud(commands: &mut Commands, asset_server: &AssetServer) {
    spawn_hud_rail(commands, asset_server, HudRail::Top, |parent| {
        spawn_hud_gauge(
            parent,
            asset_server,
            "1UP",
            HudGaugeKind::Health,
            Color::srgb(0.95, 0.39, 0.14),
            Color::srgb(0.20, 0.08, 0.04),
            178.0,
            12,
        );
        spawn_hud_gauge(
            parent,
            asset_server,
            "AMMO",
            HudGaugeKind::Ammo,
            Color::srgb(1.00, 0.55, 0.08),
            Color::srgb(0.22, 0.10, 0.03),
            218.0,
            12,
        );
        spawn_static_hud_segment(
            parent,
            asset_server,
            "LIVES 01",
            Color::srgb(0.55, 0.68, 0.68),
            132.0,
        );
        spawn_hud_value_segment(
            parent,
            asset_server,
            "KEYS",
            "00",
            KeysText,
            Color::srgb(0.64, 0.84, 0.82),
            104.0,
        );
        spawn_hud_value_segment(
            parent,
            asset_server,
            "BIO",
            "00",
            BioText,
            Color::srgb(0.62, 0.95, 0.82),
            92.0,
        );
        spawn_static_hud_segment(
            parent,
            asset_server,
            "+",
            Color::srgb(0.95, 0.76, 0.34),
            42.0,
        );
    });

    spawn_hud_rail(commands, asset_server, HudRail::Bottom, |parent| {
        spawn_hud_value_segment(
            parent,
            asset_server,
            "OBJ",
            "standby",
            ObjectiveText,
            Color::srgb(0.90, 0.84, 0.52),
            390.0,
        );
        spawn_hud_value_segment(
            parent,
            asset_server,
            "SECTION",
            "loading",
            SectionText,
            Color::srgb(0.55, 0.75, 0.92),
            360.0,
        );
        spawn_hud_segment(
            parent,
            asset_server,
            "",
            PromptText,
            Color::srgb(0.72, 0.96, 0.86),
            430.0,
        );
        spawn_hud_segment(
            parent,
            asset_server,
            "",
            NoticeText,
            Color::srgb(0.95, 0.78, 0.32),
            330.0,
        );
    });
}

enum HudRail {
    Top,
    Bottom,
}

fn spawn_hud_rail(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rail: HudRail,
    children: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let mut node = Node {
        position_type: PositionType::Absolute,
        left: px(0),
        width: percent(100),
        height: px(36),
        display: Display::Flex,
        align_items: AlignItems::Center,
        column_gap: px(8),
        padding: UiRect::horizontal(px(8)),
        border: UiRect::all(px(2)),
        ..default()
    };

    match rail {
        HudRail::Top => node.top = px(0),
        HudRail::Bottom => node.bottom = px(0),
    }

    let image = match rail {
        HudRail::Top => asset_server.load(HUD_RAIL_TOP_PATH),
        HudRail::Bottom => asset_server.load(HUD_RAIL_BOTTOM_PATH),
    };

    commands
        .spawn((
            node,
            ImageNode {
                image,
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(8.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.016, 0.018, 0.92)),
            BorderColor::all(Color::srgba(0.30, 0.42, 0.42, 0.86)),
        ))
        .with_children(children);
}

fn spawn_hud_segment<M: Component>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &'static str,
    marker: M,
    color: Color,
    width: f32,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                height: px(26),
                display: Display::Flex,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            hud_segment_image(asset_server, HUD_SEGMENT_CYAN_PATH),
            BackgroundColor(Color::srgba(0.04, 0.055, 0.055, 0.90)),
            BorderColor::all(Color::srgba(0.18, 0.85, 0.78, 0.45)),
        ))
        .with_children(|segment| {
            segment.spawn((
                Text::new(text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
                marker,
            ));
        });
}

fn spawn_hud_gauge(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &'static str,
    kind: HudGaugeKind,
    active_color: Color,
    _inactive_color: Color,
    width: f32,
    pip_count: usize,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                height: px(26),
                display: Display::Flex,
                align_items: AlignItems::Center,
                column_gap: px(7),
                padding: UiRect::horizontal(px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            hud_segment_image(asset_server, HUD_SEGMENT_CYAN_PATH),
            BackgroundColor(Color::srgba(0.04, 0.055, 0.055, 0.90)),
            BorderColor::all(Color::srgba(0.18, 0.85, 0.78, 0.45)),
        ))
        .with_children(|segment| {
            segment.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.92, 0.88)),
                Node {
                    width: px(46),
                    ..default()
                },
            ));

            segment
                .spawn(Node {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    column_gap: px(3),
                    ..default()
                })
                .with_children(|pips| {
                    for index in 0..pip_count {
                        pips.spawn((
                            Node {
                                width: px(7),
                                height: px(14),
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BackgroundColor(active_color),
                            BorderColor::all(Color::srgba(1.0, 0.72, 0.26, 0.58)),
                            HudGaugePip { kind, index },
                        ));
                    }
                });
        });
}

fn spawn_hud_value_segment<M: Component>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &'static str,
    value: &'static str,
    marker: M,
    color: Color,
    width: f32,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                height: px(26),
                display: Display::Flex,
                align_items: AlignItems::Center,
                column_gap: px(8),
                padding: UiRect::horizontal(px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            hud_segment_image(asset_server, HUD_SEGMENT_STEEL_PATH),
            BackgroundColor(Color::srgba(0.035, 0.040, 0.044, 0.90)),
            BorderColor::all(Color::srgba(0.62, 0.68, 0.64, 0.55)),
        ))
        .with_children(|segment| {
            segment.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.42, 0.54, 0.54)),
            ));
            segment.spawn((
                Text::new(value),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
                marker,
            ));
        });
}

fn spawn_static_hud_segment(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &'static str,
    color: Color,
    width: f32,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                height: px(26),
                display: Display::Flex,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            hud_segment_image(asset_server, HUD_SEGMENT_STEEL_PATH),
            BackgroundColor(Color::srgba(0.035, 0.040, 0.044, 0.90)),
            BorderColor::all(Color::srgba(0.62, 0.68, 0.64, 0.55)),
        ))
        .with_children(|segment| {
            segment.spawn((
                Text::new(text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
            ));
        });
}

fn hud_segment_image(asset_server: &AssetServer, path: &'static str) -> ImageNode {
    ImageNode {
        image: asset_server.load(path),
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(12.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

pub fn spawn_level(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level: &LevelDefinition,
    level_state: &game_core::LevelState,
    entry_id: Option<&str>,
    run_position: Option<Vec2>,
) {
    let apothecary_start =
        run_position.unwrap_or_else(|| apothecary_spawn_position(level, entry_id));
    let floor_size = visual_floor_size(level.bounds.half_extents * 2.0);

    spawn_floor_area(
        commands,
        asset_server,
        level.bounds.center,
        floor_size,
        Vec2::splat(128.0),
        -10.0,
        level_floor_tint(&level.name),
    );

    commands.spawn((
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.18), floor_size),
        Transform::from_xyz(level.bounds.center.x, level.bounds.center.y, -9.0),
        LevelEntity,
    ));

    commands.spawn((
        apothecary_sprite(asset_server),
        Transform::from_xyz(apothecary_start.x, apothecary_start.y, 10.0),
        Apothecary,
        apothecary_animation(asset_server),
        LevelEntity,
    ));

    for wall in split_walls_around_doors(&level.walls, &level.doors) {
        spawn_wall(
            commands,
            asset_server,
            wall.center,
            wall.half_extents * 2.0,
            &level.name,
        );
    }

    for decor in &level.decor {
        spawn_decor(commands, asset_server, decor);
    }

    for contaminant in &level.contaminants {
        if level_state.has_killed_contaminant(&contaminant.id) {
            continue;
        }

        spawn_contaminant(
            commands,
            asset_server,
            Some(contaminant.id.clone()),
            contaminant.position,
        );
    }

    for pickup in &level.pickups {
        if level_state.has_collected_pickup(&pickup.id) {
            continue;
        }

        spawn_pickup(
            commands,
            asset_server,
            pickup.id.clone(),
            pickup.position,
            pickup.kind.clone(),
        );
    }

    for door in &level.doors {
        let opened = level_state.has_unlocked_door(&door.id);
        let locked = !opened
            && door.starts_locked
            && !door_requirements_met(&door.clearance_id, &door.required_objectives, level_state);
        spawn_door(
            commands,
            asset_server,
            door.id.clone(),
            door.position,
            door.half_extents * 2.0,
            door.clearance_id.clone(),
            locked,
            opened,
            door.kind,
            door.required_objectives.clone(),
        );
    }

    for terminal in &level.terminals {
        spawn_terminal(
            commands,
            asset_server,
            terminal.id.clone(),
            terminal.position,
            terminal.kind.clone(),
            terminal.objective_id.clone(),
            terminal.actions.clone(),
        );
    }

    for exit in &level.exits {
        commands.spawn((
            image_sprite(
                asset_server,
                "sprites/biomech/exit_marker.png",
                exit.half_extents * 2.0,
                Color::srgba(0.72, 1.0, 0.95, 0.90),
            ),
            Transform::from_xyz(exit.position.x, exit.position.y, 2.0),
            ExitZone {
                target: exit.target.clone(),
                entry_id: exit.entry_id.clone(),
                required_objectives: exit.required_objectives.clone(),
                half_extents: exit.half_extents,
            },
            LevelEntity,
        ));
    }

    for transition in &level.transitions {
        let color = match transition.kind {
            TransitionKind::Lift => Color::srgba(0.74, 0.98, 1.0, 0.72),
            TransitionKind::Teleporter => Color::srgba(0.18, 1.0, 0.88, 0.90),
        };
        commands.spawn((
            image_sprite(
                asset_server,
                "sprites/biomech/exit_marker.png",
                transition.half_extents * 2.0,
                color,
            ),
            Transform::from_xyz(transition.position.x, transition.position.y, 2.5),
            TransitionZone {
                id: transition.id.clone(),
                target: transition.target.clone(),
                entry_id: transition.entry_id.clone(),
                kind: transition.kind,
                required_objectives: transition.required_objectives.clone(),
                required_clearance: transition.required_clearance.clone(),
                half_extents: transition.half_extents,
            },
            LevelEntity,
        ));
    }
}

pub fn apothecary_spawn_position(level: &LevelDefinition, entry_id: Option<&str>) -> Vec2 {
    entry_id
        .and_then(|entry_id| {
            level
                .entry_points
                .iter()
                .find(|entry_point| entry_point.id == entry_id)
        })
        .map(|entry_point| entry_point.position)
        .unwrap_or(level.apothecary_start)
}

fn apothecary_sprite(asset_server: &AssetServer) -> Sprite {
    let mut sprite = Sprite::from_image(asset_server.load("sprites/apothecary/walk_0.png"));
    sprite.custom_size = Some(Vec2::new(96.0, 50.0));
    sprite
}

fn apothecary_animation(asset_server: &AssetServer) -> ApothecaryAnimation {
    ApothecaryAnimation {
        frames: (0..=5)
            .map(|index| asset_server.load(format!("sprites/apothecary/walk_{index}.png")))
            .collect(),
        phase: 0.0,
        visual_offset: Vec2::ZERO,
    }
}

fn image_sprite(
    asset_server: &AssetServer,
    path: &'static str,
    size: Vec2,
    color: Color,
) -> Sprite {
    let mut sprite = Sprite::from_image(asset_server.load(path));
    sprite.custom_size = Some(size);
    sprite.color = color;
    sprite
}

fn spawn_tiled_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &'static str,
    center: Vec2,
    size: Vec2,
    tile_size: Vec2,
    z: f32,
    color: Color,
) {
    let columns = (size.x / tile_size.x).ceil() as i32;
    let rows = (size.y / tile_size.y).ceil() as i32;
    let origin = center - Vec2::new(columns as f32 * tile_size.x, rows as f32 * tile_size.y) * 0.5;

    for y in 0..rows {
        for x in 0..columns {
            let position = origin
                + Vec2::new(
                    x as f32 * tile_size.x + tile_size.x * 0.5,
                    y as f32 * tile_size.y + tile_size.y * 0.5,
                );
            commands.spawn((
                image_sprite(asset_server, path, tile_size, color),
                Transform::from_xyz(position.x, position.y, z),
                LevelEntity,
            ));
        }
    }
}

fn spawn_floor_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    size: Vec2,
    tile_size: Vec2,
    z: f32,
    color: Color,
) {
    let columns = (size.x / tile_size.x).ceil() as i32;
    let rows = (size.y / tile_size.y).ceil() as i32;
    let origin = center - Vec2::new(columns as f32 * tile_size.x, rows as f32 * tile_size.y) * 0.5;

    for y in 0..rows {
        for x in 0..columns {
            let position = origin
                + Vec2::new(
                    x as f32 * tile_size.x + tile_size.x * 0.5,
                    y as f32 * tile_size.y + tile_size.y * 0.5,
                );
            let path = FLOOR_TILE_PATHS[tile_variant_index(x, y)];
            commands.spawn((
                image_sprite(asset_server, path, tile_size, color),
                Transform::from_xyz(position.x, position.y, z),
                LevelEntity,
            ));
        }
    }
}

fn tile_variant_index(x: i32, y: i32) -> usize {
    let hash = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
    hash.unsigned_abs() as usize % FLOOR_TILE_PATHS.len()
}

fn visual_floor_size(playable_size: Vec2) -> Vec2 {
    playable_size + FLOOR_VISUAL_BLEED
}

pub fn update_level_runtime(
    level_runtime: &mut LevelRuntime,
    current_level_map: &mut CurrentLevelMap,
    level: &LevelDefinition,
    contaminant_timer: &mut ContaminantSpawnTimer,
) {
    let interval = level.spawn_interval_seconds.unwrap_or(0.0);
    level_runtime.camera_center = level.bounds.center;
    level_runtime.camera_size = level.bounds.half_extents * 2.0;
    level_runtime.dynamic_spawn_points = level.spawn_points.clone();
    level_runtime.dynamic_spawn_cursor = 0;
    level_runtime.dynamic_spawn_interval_seconds = interval;
    level_runtime.available_exits = level.exits.iter().map(|exit| exit.target.clone()).collect();
    current_level_map.level = Some(level.clone());

    if interval > 0.0 {
        contaminant_timer
            .0
            .set_duration(Duration::from_secs_f32(interval));
    }
    contaminant_timer.0.reset();
}

fn spawn_wall(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    size: Vec2,
    level_name: &str,
) {
    spawn_tiled_area(
        commands,
        asset_server,
        "sprites/biomech/tile_wall_biomech.png",
        center,
        size,
        if size.x >= size.y {
            Vec2::new(96.0, size.y.max(18.0))
        } else {
            Vec2::new(size.x.max(18.0), 96.0)
        },
        -5.0,
        level_wall_tint(level_name),
    );

    commands.spawn((
        Sprite::from_color(Color::NONE, size),
        Transform::from_xyz(center.x, center.y, -4.8),
        Wall {
            half_extents: size * 0.5,
        },
        LevelEntity,
    ));
}

fn split_walls_around_doors(
    walls: &[AxisAlignedBox],
    doors: &[DoorDefinition],
) -> Vec<AxisAlignedBox> {
    let mut split_walls = Vec::new();

    for wall in walls {
        let mut segments = vec![*wall];

        for door in doors {
            segments = segments
                .into_iter()
                .flat_map(|segment| split_wall_around_door(segment, door))
                .collect();
        }

        split_walls.extend(segments);
    }

    split_walls
}

fn split_wall_around_door(wall: AxisAlignedBox, door: &DoorDefinition) -> Vec<AxisAlignedBox> {
    let wall_is_horizontal = wall.half_extents.x >= wall.half_extents.y;
    let door_is_horizontal = door.half_extents.x >= door.half_extents.y;

    if wall_is_horizontal != door_is_horizontal || !door_overlaps_wall(wall, door) {
        return vec![wall];
    }

    const DOOR_CUT_PADDING: f32 = 4.0;
    let wall_min = wall.center - wall.half_extents;
    let wall_max = wall.center + wall.half_extents;
    let door_min = door.position - door.half_extents - Vec2::splat(DOOR_CUT_PADDING);
    let door_max = door.position + door.half_extents + Vec2::splat(DOOR_CUT_PADDING);

    if wall_is_horizontal {
        let left = wall_segment(
            Vec2::new(wall_min.x, wall_min.y),
            Vec2::new(door_min.x.min(wall_max.x), wall_max.y),
        );
        let right = wall_segment(
            Vec2::new(door_max.x.max(wall_min.x), wall_min.y),
            Vec2::new(wall_max.x, wall_max.y),
        );
        [left, right].into_iter().flatten().collect()
    } else {
        let bottom = wall_segment(
            Vec2::new(wall_min.x, wall_min.y),
            Vec2::new(wall_max.x, door_min.y.min(wall_max.y)),
        );
        let top = wall_segment(
            Vec2::new(wall_min.x, door_max.y.max(wall_min.y)),
            Vec2::new(wall_max.x, wall_max.y),
        );
        [bottom, top].into_iter().flatten().collect()
    }
}

fn door_overlaps_wall(wall: AxisAlignedBox, door: &DoorDefinition) -> bool {
    let door_bounds = AxisAlignedBox::new(door.position, door.half_extents);
    aabb_intersects(wall, door_bounds)
}

fn aabb_intersects(a: AxisAlignedBox, b: AxisAlignedBox) -> bool {
    let a_min = a.center - a.half_extents;
    let a_max = a.center + a.half_extents;
    let b_min = b.center - b.half_extents;
    let b_max = b.center + b.half_extents;

    a_min.x <= b_max.x && a_max.x >= b_min.x && a_min.y <= b_max.y && a_max.y >= b_min.y
}

fn wall_segment(min: Vec2, max: Vec2) -> Option<AxisAlignedBox> {
    let size = max - min;
    if size.x <= 2.0 || size.y <= 2.0 {
        return None;
    }

    Some(AxisAlignedBox::new((min + max) * 0.5, size * 0.5))
}

fn level_floor_tint(level_name: &str) -> Color {
    match level_name {
        "lab_access_corridor" => Color::srgba(0.22, 0.32, 0.36, 0.82),
        "triage_vault" => Color::srgba(0.30, 0.24, 0.34, 0.82),
        _ => Color::srgba(0.25, 0.30, 0.34, 0.82),
    }
}

fn level_wall_tint(level_name: &str) -> Color {
    match level_name {
        "lab_access_corridor" => Color::srgba(0.42, 0.62, 0.66, 0.92),
        "triage_vault" => Color::srgba(0.58, 0.46, 0.66, 0.92),
        _ => Color::srgba(0.50, 0.56, 0.62, 0.92),
    }
}

fn spawn_contaminant(
    commands: &mut Commands,
    asset_server: &AssetServer,
    id: Option<String>,
    position: Vec2,
) {
    commands.spawn((
        image_sprite(
            asset_server,
            "sprites/biomech/contaminant.png",
            Vec2::new(64.0, 50.0),
            Color::WHITE,
        ),
        Transform::from_xyz(position.x, position.y, 15.0),
        Contaminant {
            id,
            health: 2,
            hit_flash: Timer::from_seconds(0.0, TimerMode::Once),
        },
        contaminant_animation(asset_server),
        LevelEntity,
    ));
}

pub fn contaminant_animation(asset_server: &AssetServer) -> ContaminantAnimation {
    ContaminantAnimation {
        base_image: asset_server.load("sprites/biomech/contaminant.png"),
        stride_image: asset_server.load("sprites/biomech/contaminant_stride.png"),
        phase: 0.0,
    }
}

fn spawn_pickup(
    commands: &mut Commands,
    asset_server: &AssetServer,
    id: String,
    position: Vec2,
    kind: PickupKind,
) {
    let (path, size) = match kind {
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
    };

    commands.spawn((
        image_sprite(asset_server, path, size, Color::WHITE),
        Transform::from_xyz(position.x, position.y, 6.0),
        Pickup { id, kind },
        LevelEntity,
    ));
}

fn spawn_decor(commands: &mut Commands, asset_server: &AssetServer, decor: &DecorDefinition) {
    let (path, size, z, color) = decor_visual(decor.kind);
    let mut transform = Transform::from_xyz(decor.position.x, decor.position.y, z);
    transform.rotation = Quat::from_rotation_z(decor.rotation_degrees.to_radians());

    let mut entity = commands.spawn((
        image_sprite(asset_server, path, size, color),
        transform,
        LevelEntity,
    ));

    if decor.blocking {
        entity.insert(Wall {
            half_extents: size * 0.45,
        });
    }
}

fn decor_visual(kind: DecorKind) -> (&'static str, Vec2, f32, Color) {
    match kind {
        DecorKind::BloodDrops => (
            "sprites/biomech/decor_blood_drops.png",
            Vec2::new(54.0, 42.0),
            -1.0,
            Color::WHITE,
        ),
        DecorKind::BloodSmear => (
            "sprites/biomech/decor_blood_smear.png",
            Vec2::new(92.0, 44.0),
            -1.0,
            Color::WHITE,
        ),
        DecorKind::BloodPool => (
            "sprites/biomech/decor_blood_pool.png",
            Vec2::new(96.0, 72.0),
            -1.0,
            Color::WHITE,
        ),
        DecorKind::AcidScorch => (
            "sprites/biomech/decor_acid_scorch.png",
            Vec2::new(90.0, 70.0),
            -1.0,
            Color::WHITE,
        ),
        DecorKind::CrackedPanel => (
            "sprites/biomech/decor_cracked_panel.png",
            Vec2::new(84.0, 84.0),
            -1.0,
            Color::WHITE,
        ),
        DecorKind::LabTable => (
            "sprites/biomech/decor_lab_table.png",
            Vec2::new(112.0, 64.0),
            1.0,
            Color::WHITE,
        ),
        DecorKind::MedBed => (
            "sprites/biomech/decor_med_bed.png",
            Vec2::new(70.0, 118.0),
            1.0,
            Color::WHITE,
        ),
        DecorKind::BioTank => (
            "sprites/biomech/decor_bio_tank.png",
            Vec2::new(62.0, 96.0),
            1.0,
            Color::WHITE,
        ),
        DecorKind::SupplyCrate => (
            "sprites/biomech/decor_supply_crate.png",
            Vec2::new(58.0, 48.0),
            1.0,
            Color::WHITE,
        ),
        DecorKind::PipeCluster => (
            "sprites/biomech/decor_pipe_cluster.png",
            Vec2::new(116.0, 42.0),
            1.0,
            Color::WHITE,
        ),
        DecorKind::CorpsePile => (
            "sprites/biomech/decor_corpse_pile.png",
            Vec2::new(88.0, 74.0),
            1.0,
            Color::WHITE,
        ),
        DecorKind::FloorGrate => (
            "sprites/biomech/decor_floor_grate.png",
            Vec2::new(112.0, 82.0),
            -1.0,
            Color::WHITE,
        ),
        DecorKind::HazardFloor => (
            "sprites/biomech/decor_hazard_floor.png",
            Vec2::new(112.0, 38.0),
            -1.0,
            Color::WHITE,
        ),
    }
}

fn spawn_door(
    commands: &mut Commands,
    asset_server: &AssetServer,
    id: String,
    center: Vec2,
    size: Vec2,
    clearance_id: String,
    locked: bool,
    opened: bool,
    kind: DoorKind,
    required_objectives: Vec<String>,
) {
    let color = if opened {
        door_open_color(kind)
    } else if locked {
        door_locked_color(kind)
    } else {
        door_closed_color(kind)
    };

    let mut entity = commands.spawn((
        image_sprite(asset_server, door_sprite_path(kind), size, color),
        Transform::from_xyz(center.x, center.y, -3.0),
        Door {
            id,
            clearance_id,
            locked,
            opened,
            kind,
            required_objectives,
        },
        LevelEntity,
    ));

    if !opened {
        entity.insert(Wall {
            half_extents: size * 0.5,
        });
    }
}

fn door_requirements_met(
    clearance_id: &str,
    required_objectives: &[String],
    level_state: &game_core::LevelState,
) -> bool {
    let clearance_met = clearance_id == "open" || level_state.has_clearance(clearance_id);
    let objectives_met = required_objectives
        .iter()
        .all(|objective_id| level_state.objectives.is_complete(objective_id));

    clearance_met && objectives_met
}

fn door_sprite_path(kind: DoorKind) -> &'static str {
    match kind {
        DoorKind::Bulkhead => "sprites/biomech/door_bulkhead.png",
        DoorKind::EnergyBarrier => "sprites/biomech/door_energy_barrier.png",
    }
}

fn door_locked_color(kind: DoorKind) -> Color {
    match kind {
        DoorKind::Bulkhead => Color::WHITE,
        DoorKind::EnergyBarrier => Color::srgba(0.90, 0.35, 1.0, 1.0),
    }
}

fn door_closed_color(kind: DoorKind) -> Color {
    match kind {
        DoorKind::Bulkhead => Color::srgba(0.78, 1.0, 0.92, 0.90),
        DoorKind::EnergyBarrier => Color::srgba(0.72, 0.45, 1.0, 0.90),
    }
}

fn door_open_color(kind: DoorKind) -> Color {
    match kind {
        DoorKind::Bulkhead => Color::srgba(0.55, 0.85, 0.80, 0.42),
        DoorKind::EnergyBarrier => Color::srgba(0.20, 0.95, 1.0, 0.26),
    }
}

fn spawn_terminal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    id: String,
    position: Vec2,
    kind: TerminalKind,
    objective_id: Option<String>,
    actions: Vec<game_core::LevelEvent>,
) {
    let (path, color) = match kind {
        TerminalKind::LabAnalyzer => ("sprites/biomech/terminal_lab_analyzer.png", Color::WHITE),
        TerminalKind::ShipLog => (
            "sprites/biomech/terminal_lab_analyzer.png",
            Color::srgba(0.78, 0.88, 1.0, 1.0),
        ),
        TerminalKind::SupplyConsole => {
            ("sprites/biomech/terminal_supply_console.png", Color::WHITE)
        }
    };

    commands.spawn((
        image_sprite(asset_server, path, Vec2::new(42.0, 40.0), color),
        Transform::from_xyz(position.x, position.y, 4.0),
        Terminal {
            id,
            kind,
            objective_id,
            actions,
        },
        LevelEntity,
    ));
}
