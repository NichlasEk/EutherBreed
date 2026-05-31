use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::resources::LevelRuntime;

const CAMERA_PADDING: f32 = 80.0;

pub fn sync_camera_to_level(
    level_runtime: Res<LevelRuntime>,
    camera_query: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (mut transform, mut projection) = camera_query.into_inner();
    transform.translation.x = level_runtime.camera_center.x;
    transform.translation.y = level_runtime.camera_center.y;

    let Projection::Orthographic(orthographic) = &mut *projection else {
        return;
    };

    let camera_size = level_runtime.camera_size + Vec2::splat(CAMERA_PADDING * 2.0);
    orthographic.scaling_mode = ScalingMode::AutoMin {
        min_width: camera_size.x,
        min_height: camera_size.y,
    };
    orthographic.scale = 1.0;
}
