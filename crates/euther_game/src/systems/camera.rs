use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::components::Apothecary;
use crate::resources::CurrentLevelMap;

const CAMERA_PADDING: f32 = 80.0;
const CAMERA_VIEW_SIZE: Vec2 = Vec2::new(900.0, 520.0);

pub fn sync_camera_to_level(
    current_map: Res<CurrentLevelMap>,
    apothecary_query: Query<&Transform, (With<Apothecary>, Without<Camera2d>)>,
    camera_query: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (mut transform, mut projection) = camera_query.into_inner();

    let target = apothecary_query
        .single()
        .ok()
        .map(|apothecary| apothecary.translation.xy())
        .unwrap_or_else(|| {
            current_map
                .level
                .as_ref()
                .map(|level| level.bounds.center)
                .unwrap_or(Vec2::ZERO)
        });

    let center = current_map
        .level
        .as_ref()
        .map(|level| {
            camera_center_inside_level(target, level.bounds.center, level.bounds.half_extents)
        })
        .unwrap_or(target);

    transform.translation.x = center.x;
    transform.translation.y = center.y;

    let Projection::Orthographic(orthographic) = &mut *projection else {
        return;
    };

    let camera_size = CAMERA_VIEW_SIZE + Vec2::splat(CAMERA_PADDING * 2.0);
    orthographic.scaling_mode = ScalingMode::AutoMin {
        min_width: camera_size.x,
        min_height: camera_size.y,
    };
    orthographic.scale = 1.0;
}

fn camera_center_inside_level(target: Vec2, level_center: Vec2, level_half_extents: Vec2) -> Vec2 {
    let half_view = CAMERA_VIEW_SIZE * 0.5;
    let min = level_center - level_half_extents + half_view;
    let max = level_center + level_half_extents - half_view;

    Vec2::new(
        clamp_axis(target.x, min.x, max.x, level_center.x),
        clamp_axis(target.y, min.y, max.y, level_center.y),
    )
}

fn clamp_axis(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if min <= max {
        value.clamp(min, max)
    } else {
        fallback
    }
}
