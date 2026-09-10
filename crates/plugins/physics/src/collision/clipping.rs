use arrayvec::ArrayVec;
use glam::Vec3;

use crate::collision::{
    colliders::{CubeCollider, Edge, EdgeIndex, Face, FaceIndex},
    collision_management::{ContactManifold, ContactPoint},
    sat::{SATFeature, SATResult},
};

pub trait Clippable {
    fn get_face_from_index(&self, index: FaceIndex) -> Face;
    fn get_incident_face(&self, reference_normal: Vec3) -> Face;
}

pub trait EdgeEdgeable {
    fn get_edge_from_index(&self, index: EdgeIndex) -> Edge;
}

pub fn get_contact_manifold_from_sat<A: Clippable + EdgeEdgeable, B: Clippable + EdgeEdgeable>(
    sat: SATResult,
    shape_a: A,
    shape_b: B,
) -> ContactManifold {
    match sat.feature {
        SATFeature::EdgeEdge { .. } => edge_edge_manifold(sat, shape_a, shape_b),
        _ => clipping(sat, shape_a, shape_b),
    }
}

// I use the trait clippable, but yeah, it's not the right name, maybe I should create a third trait, but anyway
fn edge_edge_manifold<A: EdgeEdgeable, B: EdgeEdgeable>(
    sat: SATResult,
    shape_a: A,
    shape_b: B,
) -> ContactManifold {
    let SATFeature::EdgeEdge {
        edge_a_index: index_a,
        edge_b_index: index_b,
    } = sat.feature
    else {
        unreachable!("edge_edge_manifold func but sat feature is not edge edge")
    };
    let edge_a: Edge = shape_a.get_edge_from_index(index_a);
    let edge_b = shape_b.get_edge_from_index(index_b);
    let la = |s: f32| edge_a.get_a() + s * edge_a.get_direction();
    let lb = |t: f32| edge_b.get_a() + t * edge_b.get_direction();

    let r = edge_a.get_a() - edge_b.get_a();

    let a = edge_a.get_direction().dot(edge_a.get_direction());
    let b = edge_a.get_direction().dot(edge_b.get_direction());
    let c = edge_b.get_direction().dot(edge_b.get_direction());
    let d = edge_a.get_direction().dot(r);
    let e = edge_b.get_direction().dot(r);

    let determinant = a * c - b * b;

    if determinant <= 0.0 {
        // idk
        // can it happen?
    }

    let mut s = (b * e - c * d) / determinant;
    let mut t = (a * e - b * d) / determinant;

    let s_clamped = s.clamp(0.0, 1.0);
    let t_clamped = t.clamp(0.0, 1.0);

    t = (b * s_clamped + e) / c;
    s = ((b * t_clamped - d) / a).clamp(0.0, 1.0);

    let c_a = la(s);
    let c_b = lb(t);

    let p_contact = (c_a + c_b) / 2.0;

    let contact_point = ContactPoint {
        world: p_contact,
        depth: sat.mtv.length(),
    };

    let mut points = ArrayVec::new();
    points.push(contact_point);

    ContactManifold {
        points: points,
        mtv: sat.mtv,
    }
}

fn clipping<A: Clippable, B: Clippable>(sat: SATResult, shape_a: A, shape_b: B) -> ContactManifold {
    let mut is_a_ref = true;
    let reference = match sat.feature {
        SATFeature::FaceA { face_index } => shape_a.get_face_from_index(face_index),
        SATFeature::FaceB { face_index } => {
            is_a_ref = false;
            shape_b.get_face_from_index(face_index)
        }
        SATFeature::EdgeEdge { .. } => {
            unreachable!("Unreachable: clipping algorithm but feature is edge edge")
        }
    };

    let incident = if is_a_ref {
        shape_b.get_incident_face(reference.normal)
    } else {
        shape_a.get_incident_face(reference.normal)
    };

    let inward_planes = reference.get_inward_planes();

    let mut result: ArrayVec<Vec3, 8> = incident.vertices.into_iter().collect();
    let mut buffer: ArrayVec<Vec3, 8> = ArrayVec::new();
    for plane in inward_planes {
        let len = result.len();
        (0..len)
            .map(|i| (result[i], result[(i + 1) % len]))
            .for_each(|(v1, v2)| {
                let d1 = plane.distance_to_point(v1);
                let d2 = plane.distance_to_point(v2);

                fn calculate_intercepted_point(v1: Vec3, v2: Vec3, d1: f32, d2: f32) -> Vec3 {
                    let t = d1 / (d1 - d2);
                    v1 + t * (v2 - v1)
                }

                if d1 >= 0.0 && d2 >= 0.0 {
                    buffer.push(v2);
                } else if d1 >= 0.0 && d2 < 0.0 {
                    buffer.push(calculate_intercepted_point(v1, v2, d1, d2));
                } else if d1 < 0.0 && d2 > 0.0 {
                    buffer.push(calculate_intercepted_point(v1, v2, d1, d2));
                    buffer.push(v2);
                } else if d1 < 0.0 && d2 < 0.0 {
                    // do nothing
                }
            });
        result = buffer.clone();
        buffer.clear();
    }

    let contact_points = result
        .iter()
        .filter_map(|vertex| {
            let distance = reference.distance_to_point(*vertex);
            if distance > 0.0 {
                None
            } else {
                Some((vertex, distance))
            }
        })
        .map(|(vertex, distance)| ContactPoint {
            world: *vertex,
            depth: -distance,
        })
        .collect();

    ContactManifold {
        points: contact_points,
        mtv: sat.mtv,
    }
}

impl Clippable for CubeCollider {
    fn get_face_from_index(&self, index: FaceIndex) -> Face {
        let rot = self.transform.rotation.normalize();
        let pos = self.transform.position;
        let half_extents = self.transform.scale * 0.5;

        let (local_n, local_u, local_v) = match index {
            0 => (Vec3::X, Vec3::Y, Vec3::Z),  // +X
            1 => (Vec3::Y, Vec3::Z, Vec3::X),  // +Y
            2 => (Vec3::Z, Vec3::X, Vec3::Y),  // +Z
            3 => (-Vec3::X, Vec3::Z, Vec3::Y), // -X
            4 => (-Vec3::Y, Vec3::X, Vec3::Z), // -Y
            5 => (-Vec3::Z, Vec3::Y, Vec3::X), // -Z
            _ => panic!("Invalid face index"),
        };

        let h_n = local_n * half_extents;
        let h_u = local_u * half_extents;
        let h_v = local_v * half_extents;

        let local_corners = [
            h_n + h_u + h_v,
            h_n - h_u + h_v,
            h_n - h_u - h_v,
            h_n + h_u - h_v,
        ];

        let mut vertices = ArrayVec::new();
        for corner in local_corners {
            vertices.push(pos + rot * corner);
        }

        Face {
            normal: rot * local_n,
            vertices,
        }
    }

    fn get_incident_face(&self, ref_normal: Vec3) -> Face {
        let rot = self.transform.rotation.normalize();

        let local_dir = rot.conjugate() * ref_normal;

        let abs_dir = local_dir.abs();
        let face_index = if abs_dir.x > abs_dir.y && abs_dir.x > abs_dir.z {
            if local_dir.x < 0.0 { 0 } else { 3 } // Opposite sign
        } else if abs_dir.y > abs_dir.z {
            if local_dir.y < 0.0 { 1 } else { 4 }
        } else {
            if local_dir.z < 0.0 { 2 } else { 5 }
        };

        self.get_face_from_index(face_index)
    }
}

impl EdgeEdgeable for CubeCollider {
    fn get_edge_from_index(&self, index: EdgeIndex) -> Edge {
        // FUNC MADE WITH AI
        let axis_index = index / 4;
        let quadrant = index % 4;

        // 1. Decode quadrant (0..3) back to local coordinate signs
        let (sign_a, sign_b) = match quadrant {
            0 => (1.0, 1.0),   // (+, +)
            1 => (-1.0, 1.0),  // (-, +)
            2 => (-1.0, -1.0), // (-, -)
            3 => (1.0, -1.0),  // (+, -)
            _ => unreachable!(),
        };

        // 2. Account for scale on the local dimensions
        let extents = self.transform.scale * 0.5;

        // 3. Construct local endpoints along the primary axis
        let (local_start, local_end) = match axis_index {
            0 => (
                // X-aligned: sign_a applies to Y, sign_b applies to Z
                Vec3::new(-extents.x, sign_a * extents.y, sign_b * extents.z),
                Vec3::new(extents.x, sign_a * extents.y, sign_b * extents.z),
            ),
            1 => (
                // Y-aligned: sign_a applies to X, sign_b applies to Z
                Vec3::new(sign_a * extents.x, -extents.y, sign_b * extents.z),
                Vec3::new(sign_a * extents.x, extents.y, sign_b * extents.z),
            ),
            2 => (
                // Z-aligned: sign_a applies to X, sign_b applies to Y
                Vec3::new(sign_a * extents.x, sign_b * extents.y, -extents.z),
                Vec3::new(sign_a * extents.x, sign_b * extents.y, extents.z),
            ),
            _ => unreachable!(),
        };

        // 4. Transform endpoints into world space
        let rot = self.transform.rotation.normalize();
        let pos = self.transform.position;

        Edge::from_two_points(pos + rot * local_start, pos + rot * local_end)
    }
}
