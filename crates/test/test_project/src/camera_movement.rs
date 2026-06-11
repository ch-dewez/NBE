use core_components::transform::Transform;
use ecs::{component::Component, query::Query, ressource::Res};
use glam::Vec3;

use window::{glfw::Key, input::InputManager};


pub struct CameraMovementComponent {
    pub speed: f32,
    pub mouse_sensitivity: f32
}
impl Component for CameraMovementComponent {}

pub fn move_camera_system(query: Query<(&mut Transform, &CameraMovementComponent)>, input: Res<InputManager>) {
    for (mut transform, movement) in query.into_iter() {
    
        let yaw = input.mouse_delta.0 as f32 * movement.mouse_sensitivity;
        let pitch = input.mouse_delta.1 as f32 * movement.mouse_sensitivity;
    
        let epsilon = 1e-6;

        if yaw > epsilon || yaw < -epsilon {
            let yaw_rotation = glam::Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), yaw);
            transform.rotation = yaw_rotation * transform.rotation;
        }

        if pitch > epsilon || pitch < -epsilon {
            let pitch_rotation = glam::Quat::from_axis_angle(glam::Vec3::X, pitch);
            transform.rotation *= pitch_rotation;
        }


        let forward = transform.rotation.mul_vec3(Vec3::new(0., 0., -1.));
        let right = transform.rotation.mul_vec3(Vec3::new(1., 0., 0.));
        let up = transform.rotation.mul_vec3(Vec3::new(0., 1., 0.));

        let mut direction = Vec3::ZERO;

        let mut speed = movement.speed;

        if input.pressed_keys.contains(&Key::W) {
            direction += forward;
        }
        if input.pressed_keys.contains( &Key::S ) {
            direction -= forward;
        }
        if input.pressed_keys.contains( &Key::D ) {
            direction += right;
        }
        if input.pressed_keys.contains( &Key::A ) {
            direction -= right;
        }
        if input.pressed_keys.contains( &Key::Space ) {
            direction += up;
        }
        if input.pressed_keys.contains( &Key::LeftControl ) {
            direction -= up;
        }
        if input.pressed_keys.contains(&Key::LeftShift){
            speed *= 2.0;
        }

        if direction != Vec3::ZERO {
            transform.position += direction.normalize() * speed;
        }
    }
}



