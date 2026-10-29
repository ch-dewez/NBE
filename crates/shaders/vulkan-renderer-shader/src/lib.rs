#![no_std]
use glam::{Vec2, Vec3, Vec4};
// #[cfg(target_arch = "spirv")]
// use spirv_std::num_traits::Float;
use spirv_std::spirv;

const POSITIONS: [Vec2; 3] = [
    Vec2::new(0.0, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

const COLORS: [Vec3; 3] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
];

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: usize,
    #[spirv(position)] vtx_pos: &mut Vec4,
    frag_color: &mut Vec3,
) {
    *vtx_pos = Vec4::from((POSITIONS[vert_id], 0.0, 1.0));
    *frag_color = COLORS[vert_id];
}

#[spirv(fragment)]
pub fn main_fs(frag_color: Vec3, output: &mut Vec4) {
    *output = Vec4::from((frag_color, 1.0));
}
