use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use crate::components::{Apothecary, ApothecaryAnimation, Wall};
use crate::geometry::circle_hits_any_wall;

const APOTHECARY_SPEED: f32 = 260.0;
const APOTHECARY_RADIUS: f32 = 22.0;
const APOTHECARY_WALK_FPS: f32 = 11.0;
const APOTHECARY_WALK_SWAY_RADIANS: f32 = 0.085;
const APOTHECARY_WALK_SIDE_OFFSET: f32 = 3.2;

pub fn move_apothecary(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    wall_query: Query<(&Transform, &Wall), Without<Apothecary>>,
    mut query: Query<&mut Transform, With<Apothecary>>,
) {
    let mut movement = Vec2::ZERO;

    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        movement.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        movement.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        movement.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        movement.x += 1.0;
    }

    if movement == Vec2::ZERO {
        return;
    }

    let delta = movement.normalize() * APOTHECARY_SPEED * time.delta_secs();

    for mut transform in &mut query {
        let current = transform.translation.xy();
        let next = Vec2::new(
            (current.x + delta.x).clamp(-430.0, 430.0),
            (current.y + delta.y).clamp(-230.0, 230.0),
        );

        if !circle_hits_any_wall(next, APOTHECARY_RADIUS, &wall_query) {
            transform.translation.x = next.x;
            transform.translation.y = next.y;
            continue;
        }

        let x_only = Vec2::new(next.x, current.y);
        if !circle_hits_any_wall(x_only, APOTHECARY_RADIUS, &wall_query) {
            transform.translation.x = x_only.x;
        }

        let y_only = Vec2::new(transform.translation.x, next.y);
        if !circle_hits_any_wall(y_only, APOTHECARY_RADIUS, &wall_query) {
            transform.translation.y = y_only.y;
        }
    }
}

pub fn aim_apothecary(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    mut apothecary_query: Query<&mut Transform, With<Apothecary>>,
) {
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_position) = window_query.cursor_position() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for mut transform in &mut apothecary_query {
        let direction = world_position - transform.translation.xy();

        if direction.length_squared() > 0.001 {
            transform.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
        }
    }
}

pub fn animate_apothecary_walk(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &mut ApothecaryAnimation), With<Apothecary>>,
) {
    let walking = input.pressed(KeyCode::KeyW)
        || input.pressed(KeyCode::ArrowUp)
        || input.pressed(KeyCode::KeyS)
        || input.pressed(KeyCode::ArrowDown)
        || input.pressed(KeyCode::KeyA)
        || input.pressed(KeyCode::ArrowLeft)
        || input.pressed(KeyCode::KeyD)
        || input.pressed(KeyCode::ArrowRight);

    for (mut transform, mut sprite, mut animation) in &mut query {
        if animation.frames.is_empty() {
            continue;
        }

        transform.translation.x -= animation.visual_offset.x;
        transform.translation.y -= animation.visual_offset.y;
        animation.visual_offset = Vec2::ZERO;

        if walking {
            animation.phase += time.delta_secs() * APOTHECARY_WALK_FPS;
        } else {
            animation.phase = 0.0;
        }

        let frame = animation.phase as usize % animation.frames.len();
        sprite.image = animation.frames[frame].clone();

        if walking {
            let stride =
                (animation.phase * std::f32::consts::TAU / animation.frames.len() as f32).sin();
            let side = transform.rotation * Vec3::Y;
            animation.visual_offset = side.xy() * stride * APOTHECARY_WALK_SIDE_OFFSET;
            transform.translation.x += animation.visual_offset.x;
            transform.translation.y += animation.visual_offset.y;
            transform.rotation *= Quat::from_rotation_z(stride * APOTHECARY_WALK_SWAY_RADIANS);
            transform.scale = Vec3::new(1.0, 1.0 + stride.abs() * 0.045, 1.0);
        } else {
            transform.scale = Vec3::ONE;
        }
    }
}

pub fn quit_on_escape(input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

pub fn toggle_fullscreen_on_f11(
    input: Res<ButtonInput<KeyCode>>,
    mut window_query: Single<&mut Window, With<PrimaryWindow>>,
) {
    if !input.just_pressed(KeyCode::F11) {
        return;
    }

    window_query.mode = match window_query.mode {
        WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        _ => WindowMode::Windowed,
    };
}
