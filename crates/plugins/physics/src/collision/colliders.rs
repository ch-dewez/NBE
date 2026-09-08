use std::mem::MaybeUninit;

use core_components::transform::Transform;
use ecs::component::Component;
use glam::{Quat, Vec3};

use crate::collision::sat::{
    Axis, AxisType, SATable, SATableForEachAxesClosure, SATableForEachVecClosure,
};

#[derive(Clone)]
pub enum Collider {
    CubeCollider(CubeCollider),
}

impl From<CubeCollider> for Collider {
    fn from(value: CubeCollider) -> Self {
        Self::CubeCollider(value)
    }
}

#[derive(Default, Clone)]
pub struct CubeCollider {
    pub transform: Transform, // position is offset from local origin
}

impl CubeCollider {
    pub(crate) fn get_world_space(&self, transform: &Transform) -> CubeCollider {
        // 1. Scale the local offset by parent scale
        let scaled_offset = self.transform.position * transform.scale;

        // 2. Rotate the scaled offset by parent rotation
        let rotated_offset = transform.rotation * scaled_offset;

        // 3. Translate by parent position
        let world_position = transform.position + rotated_offset;

        // 4. Combine rotations (Parent * Local)
        let world_rotation = transform.rotation * self.transform.rotation;

        // 5. Combine scales
        let world_scale = self.transform.scale * transform.scale;

        CubeCollider {
            transform: Transform {
                position: world_position,
                rotation: world_rotation,
                scale: world_scale,
            },
        }
    }
}

impl SATable for CubeCollider {
    fn for_all_axes<'a>(
        &self,
        other: &dyn SATable,
        all_axis_closure: super::sat::SATableForEachAxesClosure<'a>,
    ) {
        self.for_each_faces_axes(&mut |my_axis: Vec3| {
            // self face axis
            if all_axis_closure(Axis {
                axis: my_axis,
                axis_type: AxisType::Face,
            }) {
                return true;
            }

            let mut should_return: bool = false;

            // add edge edge axis
            other.for_each_edges_axes(&mut |other_axis| {
                // my_axis are face, but for a cube, face and edge axis are the same
                let cross = my_axis.cross(other_axis);
                if let Some(normalized_cross) = cross.try_normalize() {
                    if all_axis_closure(Axis {
                        axis: cross,
                        axis_type: AxisType::Edge,
                    }) {
                        should_return = true;
                        return true;
                    }
                }

                false
            });

            if should_return { true } else { false }
        });

        // add other face axis
        other.for_each_faces_axes(&mut |other_axis| {
            if all_axis_closure(Axis {
                axis: other_axis,
                axis_type: AxisType::Face,
            }) {
                return true;
            }

            false
        });
    }

    fn for_each_faces_axes<'a>(&self, closure: SATableForEachVecClosure<'a>) {
        let rotation = self.transform.rotation.normalize();
        let axes = [rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z];

        for axis in axes.iter() {
            if closure(*axis) {
                break;
            }
        }
    }

    fn for_each_edges_axes<'a>(&self, closure: SATableForEachVecClosure) {
        // For a cube, edges axes are the same as faces axes
        self.for_each_faces_axes(closure)
    }

    // optimized because it's cube
    fn project_onto_axis(&self, axis: Vec3) -> (f32, f32) {
        let center = axis.dot(self.transform.position);

        let half_extents = self.transform.scale * 0.5;
        let mut axes = [Vec3::default(); 3]; // [u_x, u_y, u_z]
        let mut index = 0;
        self.for_each_faces_axes(&mut |axis| {
            axes[index] = axis;
            index += 1;
            false
        });

        // Calculate radius along the axis
        let radius = half_extents.x * axis.dot(axes[0]).abs()
            + half_extents.y * axis.dot(axes[1]).abs()
            + half_extents.z * axis.dot(axes[2]).abs();

        (center - radius, center + radius)
    }
}

#[derive(Default)]
pub struct Colliders {
    pub colliders: Vec<Collider>,
}
impl Component for Colliders {}

impl<T> From<T> for Colliders
where
    T: Into<Collider>,
{
    fn from(value: T) -> Self {
        Self {
            colliders: vec![value.into()],
        }
    }
}
impl<T> From<&[T]> for Colliders
where
    T: Into<Collider> + Clone,
{
    fn from(value: &[T]) -> Self {
        Self {
            colliders: value.iter().map(|e| e.clone().into()).collect(),
        }
    }
}

impl Colliders {
    pub fn add_collider(&mut self, collider: Collider) {
        self.colliders.push(collider);
    }
}
