use std::ops::Deref;

use arrayvec::ArrayVec;
use core_components::transform::Transform;
use ecs::component::Component;
use glam::{Mat3, Vec3};

pub(crate) type FaceIndex = usize;
pub(crate) type EdgeIndex = usize;

#[derive(Debug, Clone)]
pub(crate) struct Face {
    pub normal: Vec3,
    pub vertices: ArrayVec<Vec3, 4>,
}

impl Face {
    pub fn get_inward_planes(&self) -> ArrayVec<Plane, 4> {
        let len = self.vertices.len();
        (0..len)
            .map(|i| (self.vertices[i], self.vertices[(i + 1) % len]))
            .map(|(curr, next)| {
                let inward_normal = self.normal.cross(next - curr).normalize_or_zero();
                Plane::from_point_and_normal(curr, inward_normal)
            })
            .collect()
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point - self.vertices[0])
    }
}

pub(crate) struct Plane {
    pub normal: Vec3,
    pub d: f32,
}

impl Plane {
    pub fn from_point_and_normal(point: Vec3, normal: Vec3) -> Self {
        Self {
            normal,
            d: -normal.dot(point),
        }
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.d
    }
}

pub(crate) struct Edge {
    a: Vec3,
    b: Vec3,

    // invariant, direction = b - a
    direction: Vec3,
}

impl Edge {
    pub fn from_two_points(a: Vec3, b: Vec3) -> Self {
        Edge {
            a,
            b,

            direction: b - a,
        }
    }

    #[allow(dead_code)]
    pub fn from_point_and_direction(a: Vec3, direction: Vec3) -> Self {
        Edge {
            a,
            b: a + direction,

            direction,
        }
    }

    pub fn get_a(&self) -> Vec3 {
        self.a
    }

    #[allow(dead_code)]
    pub fn get_b(&self) -> Vec3 {
        self.b
    }

    pub fn get_direction(&self) -> Vec3 {
        self.direction
    }
}

pub(crate) trait InertiaTensorGettable {
    fn get_center_of_mass(&self, external_scale: Vec3) -> Vec3;
    fn get_inertia_tensor_local(&self, external_scale: Vec3) -> Mat3;
    fn get_inertia_tensor_rotation(&self, external_scale: Vec3) -> Mat3;
    fn get_inertia_tensor_shift(&self, relative_center_of_mass: Vec3, external_scale: Vec3)
    -> Mat3;
    fn get_volume(&self, external_scale: Vec3) -> f32;
    fn get_mass(&self, external_scale: Vec3) -> f32;
}

pub trait ColliderTrait: InertiaTensorGettable {}
impl<T> ColliderTrait for T where T: InertiaTensorGettable {}

#[derive(Clone)]
pub enum Collider {
    CubeCollider(CubeCollider),
}

impl From<CubeCollider> for Collider {
    fn from(value: CubeCollider) -> Self {
        Self::CubeCollider(value)
    }
}

impl Collider {
    pub fn as_collider_trait(&self) -> &dyn ColliderTrait {
        match self {
            Collider::CubeCollider(collider) => collider,
        }
    }
}

#[derive(Default, Clone)]
pub struct CubeCollider {
    pub transform: Transform, // position is offset from local origin
    pub density: Option<f32>,
}

impl CubeCollider {
    pub(crate) fn get_world_space(&self, transform: &Transform) -> CubeCollider {
        // FUNCTION (math) MADE BY AI

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
            density: self.density,
        }
    }
}

impl InertiaTensorGettable for CubeCollider {
    fn get_center_of_mass(&self, external_scale: Vec3) -> Vec3 {
        self.transform.position * external_scale
    }

    fn get_inertia_tensor_local(&self, external_scale: Vec3) -> Mat3 {
        let mass = self.get_mass(external_scale);
        let effective_scale = self.transform.scale * external_scale;
        let length_squared = effective_scale * effective_scale;
        Mat3::from_diagonal(
            Vec3::new(
                length_squared.y + length_squared.z,
                length_squared.x + length_squared.z,
                length_squared.x + length_squared.y,
            ) * mass
                / 12.0,
        )
    }

    fn get_inertia_tensor_shift(
        &self,
        relative_center_of_mass: Vec3,
        external_scale: Vec3,
    ) -> Mat3 {
        let d = self.get_center_of_mass(external_scale) - relative_center_of_mass;
        let d2 = d * d;
        let m = self.get_mass(external_scale);

        m * Mat3::from_cols(
            Vec3::new(d2.y + d2.z, -d.x * d.y, -d.x * d.z),
            Vec3::new(-d.x * d.y, d2.x + d2.z, -d.y * d.z),
            Vec3::new(-d.x * d.z, -d.y * d.z, d2.x + d2.y),
        )
    }

    fn get_inertia_tensor_rotation(&self, external_scale: Vec3) -> Mat3 {
        let rotation = self.transform.get_rotation_matrix();
        rotation * self.get_inertia_tensor_local(external_scale) * rotation.transpose()
    }

    fn get_volume(&self, external_scale: Vec3) -> f32 {
        (self.transform.scale * external_scale).element_product()
    }

    fn get_mass(&self, external_scale: Vec3) -> f32 {
        self.get_volume(external_scale) * self.density.unwrap_or(1.0)
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

    pub fn get_local_inertia_tensor(&self, external_scale: Vec3) -> Mat3 {
        let mut total_mass = 0.0;
        let mut center_of_mass = Vec3::ZERO;
        for collider in self.colliders.iter() {
            let collider = collider.as_collider_trait();
            let mass = collider.get_mass(external_scale);
            total_mass += mass;
            let com = collider.get_center_of_mass(external_scale);
            center_of_mass += com * mass;
        }

        if total_mass > 0.0 {
            center_of_mass /= total_mass;
        }

        let mut i_total: Mat3 = Mat3::ZERO;
        for collider in self.colliders.iter() {
            let collider = collider.as_collider_trait();
            let i_rot = collider.get_inertia_tensor_rotation(external_scale);
            let i_shift = collider.get_inertia_tensor_shift(center_of_mass, external_scale);

            let i_world = i_rot + i_shift;

            i_total += i_world;
        }

        i_total
    }

    pub fn get_local_inv_inertia_tensor(&self, external_scale: Vec3) -> Mat3 {
        self.get_local_inertia_tensor(external_scale).inverse()
    }
}
