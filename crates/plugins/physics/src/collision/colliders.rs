use arrayvec::ArrayVec;
use core_components::transform::Transform;
use ecs::component::Component;
use glam::Vec3;

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
                let inward_normal = self.normal.cross(next - curr);
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
        }
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
