use glam::Vec3;

use crate::collision::{
    clipping::{Clippable, EdgeEdgeable, get_contact_manifold_from_sat},
    colliders::{CubeCollider, EdgeIndex, FaceIndex},
    collision_management::ContactManifold,
};

pub type SATableForEachAxesClosure<'a> = &'a mut dyn FnMut(SATAxis) -> bool;
pub trait SATable {
    fn for_all_axes<'a>(&self, other: &dyn SATable, closure: SATableForEachAxesClosure<'a>);
    fn project_onto_axis(&self, axis: Vec3) -> (f32, f32);
    fn get_face_index_from_axis_index(&self, axis_index: AxisIndex, sign: bool) -> usize; // sign == true : + sign == false: -
    fn get_edge_index_from_axis_index(&self, axis_index: AxisIndex, mtv: Vec3) -> usize;

    fn for_each_faces_axes<'a>(&self, is_a: bool, closure: SATableForEachAxesClosure<'a>);
    fn for_each_edges_axes<'a>(&self, closure: SATableForEachAxesClosure<'a>);
}

#[derive(Debug, Clone, Copy)]
pub enum SATFeature {
    FaceA {
        face_index: FaceIndex,
    },
    FaceB {
        face_index: FaceIndex,
    },
    EdgeEdge {
        edge_a_index: EdgeIndex,
        edge_b_index: EdgeIndex,
    },
}

impl SATFeature {
    pub fn from_sat_axis_feature<A: SATable, B: SATable>(
        feature: SATAxisFeature,
        shape_a: &A,
        shape_b: &B,
        sign: bool,
        mtv: Vec3,
    ) -> Self {
        match feature {
            SATAxisFeature::FaceA { axis_index } => Self::FaceA {
                face_index: shape_a.get_face_index_from_axis_index(axis_index, !sign),
            },
            SATAxisFeature::FaceB { axis_index } => Self::FaceB {
                face_index: shape_b.get_face_index_from_axis_index(axis_index, sign),
            },
            SATAxisFeature::EdgeEdge {
                axis_index_a,
                axis_index_b,
            } => Self::EdgeEdge {
                edge_a_index: shape_a.get_edge_index_from_axis_index(axis_index_a, mtv),
                edge_b_index: shape_b.get_edge_index_from_axis_index(axis_index_b, mtv),
            },
            SATAxisFeature::Edge { axis_index: _ } => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SATResult {
    pub mtv: Vec3,
    pub feature: SATFeature,
}

pub type AxisIndex = usize;

#[derive(Debug, Copy, Clone)]
pub enum SATAxisFeature {
    FaceA {
        axis_index: AxisIndex,
    },
    FaceB {
        axis_index: AxisIndex,
    },
    EdgeEdge {
        axis_index_a: AxisIndex,
        axis_index_b: AxisIndex,
    },
    // Only used in SATable trait implementations to gerenate EdgeEdge, not used after for the manifold, should not happen, can panic if exist after sat
    Edge {
        axis_index: AxisIndex,
    },
}

#[derive(Clone)]
pub struct SATAxis {
    pub axis: Vec3,
    pub feature: SATAxisFeature,
}

impl From<SATAxis> for Vec3 {
    fn from(value: SATAxis) -> Self {
        value.axis
    }
}

pub fn apply_sat_and_clipping<
    A: SATable + Clippable + EdgeEdgeable + Clone,
    B: SATable + Clippable + EdgeEdgeable + Clone,
>(
    shape_a: A,
    shape_b: B,
) -> Option<ContactManifold> {
    let sat_result = sat(shape_a.clone(), shape_b.clone());
    if let Some(result) = sat_result {
        println!("SATResult: {:?}", result);
        let contact = get_contact_manifold_from_sat(result, shape_a, shape_b);
        println!("Contact Manifold: {:?}", contact);
        Some(contact)
    } else {
        None
    }
}

pub fn sat<A: SATable, B: SATable>(shape_a: A, shape_b: B) -> Option<SATResult> {
    let mut overlap = f32::MAX;
    let mut overlap_axis = Vec3::ZERO;
    let mut current_feature: SATAxisFeature = SATAxisFeature::FaceA { axis_index: 0 }; // dummy value

    let mut intersect: bool = true;
    let mut sign = false;

    shape_a.for_all_axes(&shape_b, &mut |axis: SATAxis| {
        let (min1, max1) = shape_a.project_onto_axis(axis.axis);
        let (min2, max2) = shape_b.project_onto_axis(axis.axis);

        if max1 < min2 || max2 < min1 {
            intersect = false;
            return true;
        }

        let overlap1 = max1 - min2;
        let overlap2 = max2 - min1;
        let (current_overlap, current_axis, current_sign) = if overlap1 < overlap2 {
            (overlap1, -axis.axis, true)
        } else {
            (overlap2, axis.axis, false)
        };

        if overlap > current_overlap {
            overlap = current_overlap;
            overlap_axis = current_axis;
            current_feature = axis.feature;
            sign = current_sign;
        }
        return false;
    });

    if intersect {
        let mtv = overlap_axis * overlap;
        Some(SATResult {
            mtv: overlap_axis * overlap,
            feature: SATFeature::from_sat_axis_feature(
                current_feature,
                &shape_a,
                &shape_b,
                sign,
                mtv,
            ),
        })
    } else {
        None
    }
}

impl SATable for CubeCollider {
    fn for_all_axes<'a>(
        &self,
        other: &dyn SATable,
        all_axis_closure: super::sat::SATableForEachAxesClosure<'a>,
    ) {
        // self face axis
        self.for_each_faces_axes(true, &mut |my_axis| {
            if all_axis_closure(my_axis) {
                true
            } else {
                false
            }
        });

        // other face
        other.for_each_faces_axes(false, &mut |other_axis| {
            if all_axis_closure(other_axis) {
                true
            } else {
                false
            }
        });

        // edge edge
        self.for_each_edges_axes(&mut |my_axis| {
            let mut should_return: bool = false;
            let SATAxisFeature::Edge {
                axis_index: edge_index,
            } = my_axis.feature
            else {
                unreachable!("SAT: edge feature is not an edge")
            };

            // add edge edge axis
            other.for_each_edges_axes(&mut |other_axis| {
                let SATAxisFeature::Edge {
                    axis_index: other_edge_index,
                } = other_axis.feature
                else {
                    unreachable!("SAT: edge feature is not an edge")
                };

                let cross = my_axis.axis.cross(other_axis.axis);
                if let Some(normalized_cross) = cross.try_normalize() {
                    if all_axis_closure(SATAxis {
                        axis: normalized_cross,
                        feature: SATAxisFeature::EdgeEdge {
                            axis_index_a: edge_index,
                            axis_index_b: other_edge_index,
                        },
                    }) {
                        should_return = true;
                        return true;
                    }
                }

                false
            });

            if should_return { true } else { false }
        });
    }

    fn for_each_faces_axes<'a>(&self, is_a: bool, closure: SATableForEachAxesClosure<'a>) {
        let rotation = self.transform.rotation.normalize();
        let axes = [rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z];

        for (index, axis) in axes.iter().enumerate() {
            if closure(SATAxis {
                axis: *axis,
                feature: if is_a {
                    SATAxisFeature::FaceA { axis_index: index }
                } else {
                    SATAxisFeature::FaceB { axis_index: index }
                },
            }) {
                break;
            }
        }
    }

    fn for_each_edges_axes<'a>(&self, closure: SATableForEachAxesClosure<'a>) {
        // For a cube, edges axes are the same as faces axes
        self.for_each_faces_axes(true, &mut |axis| {
            let axis_index = if let SATAxisFeature::FaceA { axis_index: index } = axis.feature {
                index
            } else if let SATAxisFeature::FaceB { axis_index: index } = axis.feature {
                index
            } else {
                unreachable!("face does not have face sat axis feature")
            };

            closure(SATAxis {
                axis: axis.axis,
                feature: SATAxisFeature::Edge { axis_index },
            })
        })
    }

    fn get_face_index_from_axis_index(&self, axis_index: AxisIndex, sign: bool) -> usize {
        // add 3 if sign is +
        axis_index + sign as usize * 3
    }

    fn get_edge_index_from_axis_index(&self, axis_index: AxisIndex, mtv: Vec3) -> usize {
        // FUNCTION MADE WITH AI
        // 1. Express separation normal in cube local space
        let local_n = self.transform.rotation.conjugate() * mtv.normalize();

        // 2. Read signs of the two orthogonal components
        let (sign_a, sign_b) = match axis_index {
            0 => (local_n.y >= 0.0, local_n.z >= 0.0), // X-aligned: inspect Y, Z
            1 => (local_n.x >= 0.0, local_n.z >= 0.0), // Y-aligned: inspect X, Z
            2 => (local_n.x >= 0.0, local_n.y >= 0.0), // Z-aligned: inspect X, Y
            _ => unreachable!(),
        };

        // 3. Map signs to a 0..3 quadrant ID
        let quadrant = match (sign_a, sign_b) {
            (true, true) => 0,   // (+, +)
            (false, true) => 1,  // (-, +)
            (false, false) => 2, // (-, -)
            (true, false) => 3,  // (+, -)
        };

        // 4. Combine group base (0, 4, or 8) with quadrant offset
        (axis_index * 4) + quadrant
    }

    // optimized because it's cube
    fn project_onto_axis(&self, axis: Vec3) -> (f32, f32) {
        let center = axis.dot(self.transform.position);

        let half_extents = self.transform.scale * 0.5;
        let mut axes = [Vec3::default(); 3]; // [u_x, u_y, u_z]
        let mut index = 0;
        self.for_each_faces_axes(true, &mut |axis| {
            axes[index] = axis.axis;
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
