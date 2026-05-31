use bevy::prelude::*;
use game_core::AxisAlignedBox;

use crate::components::Wall;

pub fn circle_hits_any_wall(
    center: Vec2,
    radius: f32,
    wall_query: &Query<(&Transform, &Wall)>,
) -> bool {
    wall_query.iter().any(|(transform, wall)| {
        game_core::circle_intersects_aabb(
            center,
            radius,
            AxisAlignedBox::new(transform.translation.xy(), wall.half_extents),
        )
    })
}
