use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct AxisAlignedBox {
    pub center: Vec2,
    pub half_extents: Vec2,
}

impl AxisAlignedBox {
    pub const fn new(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            center,
            half_extents,
        }
    }
}

pub fn circle_intersects_aabb(circle_center: Vec2, radius: f32, wall: AxisAlignedBox) -> bool {
    let closest = circle_center.clamp(
        wall.center - wall.half_extents,
        wall.center + wall.half_extents,
    );

    circle_center.distance_squared(closest) < radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_intersects_box_edge() {
        let wall = AxisAlignedBox::new(Vec2::ZERO, Vec2::new(10.0, 10.0));

        assert!(circle_intersects_aabb(Vec2::new(14.0, 0.0), 5.0, wall));
    }

    #[test]
    fn circle_outside_box_misses() {
        let wall = AxisAlignedBox::new(Vec2::ZERO, Vec2::new(10.0, 10.0));

        assert!(!circle_intersects_aabb(Vec2::new(20.0, 0.0), 5.0, wall));
    }

    #[test]
    fn circle_inside_box_hits() {
        let wall = AxisAlignedBox::new(Vec2::ZERO, Vec2::new(10.0, 10.0));

        assert!(circle_intersects_aabb(Vec2::new(0.0, 0.0), 5.0, wall));
    }
}
