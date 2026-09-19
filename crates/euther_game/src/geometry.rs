use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use game_core::AxisAlignedBox;

use crate::components::Wall;

pub fn circle_hits_any_wall<F: QueryFilter>(
    center: Vec2,
    radius: f32,
    wall_query: &Query<(&Transform, &Wall), F>,
) -> bool {
    wall_query.iter().any(|(transform, wall)| {
        game_core::circle_intersects_aabb(
            center,
            radius,
            AxisAlignedBox::new(transform.translation.xy(), wall.half_extents),
        )
    })
}

/// Fraction of the first intersection along a finite segment (including starts inside).
pub fn segment_box_hit(from: Vec2, to: Vec2, center: Vec2, half: Vec2) -> Option<f32> {
    let delta = to - from;
    let min = center - half;
    let max = center + half;
    let mut enter = 0.0_f32;
    let mut leave = 1.0_f32;
    for axis in 0..2 {
        if delta[axis].abs() < 1e-6 {
            if from[axis] < min[axis] || from[axis] > max[axis] {
                return None;
            }
        } else {
            let a = (min[axis] - from[axis]) / delta[axis];
            let b = (max[axis] - from[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter > leave {
                return None;
            }
        }
    }
    Some(enter)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segment_intersection_handles_thin_walls_parallel_misses_and_inside_starts() {
        assert_eq!(
            segment_box_hit(
                Vec2::new(-100.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::ZERO,
                Vec2::new(2.0, 20.0)
            ),
            Some(0.49)
        );
        assert_eq!(
            segment_box_hit(
                Vec2::new(-100.0, 30.0),
                Vec2::new(100.0, 30.0),
                Vec2::ZERO,
                Vec2::new(2.0, 20.0)
            ),
            None
        );
        assert_eq!(
            segment_box_hit(Vec2::ZERO, Vec2::X * 100.0, Vec2::ZERO, Vec2::splat(10.0)),
            Some(0.0)
        );
        assert_eq!(
            segment_box_hit(
                Vec2::X * 50.0,
                Vec2::X * 100.0,
                Vec2::ZERO,
                Vec2::splat(10.0)
            ),
            None
        );
    }
}
