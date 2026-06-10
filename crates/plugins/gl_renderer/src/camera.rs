use ecs::{component::Component, event::EventReader, query::Query};
use glam::{Mat4};
use window::glfw::WindowEvent;

#[derive(Default)]
pub struct Camera {
    pub projection: Mat4
}
impl Camera {
    pub fn new_perspective_default(aspect_ratio: f32) -> Self{
        Self::new_perspective(45.0, aspect_ratio, 0.1, 100.)
    }

    pub fn new_perspective(
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32
    ) -> Self {
        Self {
            projection: glam::Mat4::perspective_rh_gl(fov_y_radians, aspect_ratio, z_near, z_far)
        }
    }
}

// USE EVENT TO RECALCULATE PROJ MATRIX ON WINDOW RESIZE
pub fn project_update_on_resize(query: Query<&mut Camera>, mut window_event: EventReader<window::window::EcsWindowEvent>){
    for mut camera in query.clone().into_iter(){
        for event in window_event.iter(){
            match event.0{
                WindowEvent::Size(width, height) | WindowEvent::FramebufferSize(width, height) => {
                    let aspect_ratio = width as f32 / height as f32;
                    *camera = Camera::new_perspective_default(aspect_ratio);
                },
                _ => {}
            }
        }
    }
}

impl Component for Camera {}
