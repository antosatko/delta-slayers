use bevy::prelude::*;

pub(crate) const TILE_RADIUS_METERS: f32 = 6.0;

pub(super) const HEX_CORNER_COUNT: usize = 6;

pub(super) const HEX_NEIGHBORS: [(i32, i32); 6] =
    [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

pub(crate) fn hex_center(q: i32, r: i32, size: f32) -> Vec2 {
    Vec2::new(
        3.0_f32.sqrt() * size * (q as f32 + r as f32 * 0.5),
        1.5 * size * r as f32,
    )
}

pub(super) fn hex_corner(center: Vec2, size: f32, index: usize) -> Vec2 {
    let angle = std::f32::consts::FRAC_PI_6 + index as f32 * std::f32::consts::PI / 3.0;
    center + Vec2::new(angle.cos(), angle.sin()) * size
}

pub(crate) fn world_to_axial(point: Vec2, size: f32) -> (i32, i32) {
    let q = (3.0_f32.sqrt() / 3.0 * point.x - point.y / 3.0) / size;
    let r = (2.0 / 3.0 * point.y) / size;
    let x = q;
    let z = r;
    let y = -x - z;

    let mut rx = x.round();
    let ry = y.round();
    let mut rz = z.round();
    let dx = (rx - x).abs();
    let dy = (ry - y).abs();
    let dz = (rz - z).abs();
    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy > dz {
        rz = -rx - ry;
    }
    (rx as i32, rz as i32)
}
