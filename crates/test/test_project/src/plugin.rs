use std::rc::Rc;

use core_components::transform::Transform;
use ecs::{
    command::Command,
    event::EventReader,
    ressource::{Res, ResMut},
    system::SystemParam,
};
use engine::{app_command::StopAppCommand, plugin::Plugin};
use gl_renderer::{camera::Camera, default_shaders::DefaultShaders};
use glam::{Quat, Vec3, Vec4};
use opengl::{
    context::OpenGlContext,
    material::{Material, MaterialComponent, MaterialTemplate},
    mesh::{Mesh, MeshComponent},
    program::Program,
};
use window::{
    glfw::WindowEvent,
    input::InputManager,
    window::{EcsWindowEvent, GLFWWindowRes},
};

use crate::{
    camera_movement::{CameraMovementComponent, move_camera_system},
    movement::{self, Velocity},
};

fn quit_app(
    mut command: ResMut<Command>,
    input: Res<InputManager>,
    mut window_events: EventReader<EcsWindowEvent>,
) {
    if input.pressed_keys.contains(&window::glfw::Key::Escape) {
        command.0.push(Box::new(StopAppCommand {}));
    }
    for event in window_events.iter() {
        // Not sure what is this
        if let WindowEvent::Close = event.0 {
            command.0.push(Box::new(StopAppCommand {}));
        }
    }
}

#[derive(Default)]
pub struct GamePlugin {}

impl Plugin for GamePlugin {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {
        let vertices: Box<[Vec3]> = Box::new([
            // Front face (z = 0.5)
            Vec3::new(-0.5, -0.5, 0.5), // 0: Bottom-left-front
            Vec3::new(0.5, -0.5, 0.5),  // 1: Bottom-right-front
            Vec3::new(0.5, 0.5, 0.5),   // 2: Top-right-front
            Vec3::new(-0.5, 0.5, 0.5),  // 3: Top-left-front
            // Back face (z = -0.5)
            Vec3::new(-0.5, -0.5, -0.5), // 4: Bottom-left-back
            Vec3::new(0.5, -0.5, -0.5),  // 5: Bottom-right-back
            Vec3::new(0.5, 0.5, -0.5),   // 6: Top-right-back
            Vec3::new(-0.5, 0.5, -0.5),  // 7: Top-left-back
        ]);

        let indices: Box<[u32]> = Box::new([
            // Front face
            0, 1, 2, 2, 3, 0, // Right face
            1, 5, 6, 6, 2, 1, // Back face
            5, 4, 7, 7, 6, 5, // Left face
            4, 0, 3, 3, 7, 4, // Top face
            3, 2, 6, 6, 7, 3, // Bottom face
            4, 5, 1, 1, 0, 4,
        ]);

        let gl = Res::<OpenGlContext>::retrieve_no_local(&context.application.world)
            .expect("Couldn't get open gl context");
        //
        //let graphics_ressource = Res::<GraphicsRessourceManager>::retrieve_no_local(&world).expect("Couldn't get Graphics Ressource manager");

        let mut mesh = Mesh::new(vertices, indices);
        mesh.move_to_gpu(&gl);
        let mesh = Rc::new(mesh);

        let program = Rc::new(Program::new(
            DefaultShaders::DefaultVert.code(),
            DefaultShaders::DefaultFrag.code(),
            &gl,
        ));

        let color = Vec4::new(1.0, 0.3, 0.0, 1.0);

        let material_template = Rc::new(MaterialTemplate::new(&gl, program, size_of::<Vec4>()));

        let material = Rc::new(Material::new(material_template, &gl));
        material.update_data(&gl, &color);

        let mat_comp = MaterialComponent(material.clone());
        let mesh_comp = MeshComponent(mesh.clone());
        let transform = Transform::new(
            Vec3::new(0.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::new(1.0, 1.0, 1.0),
        );
        let velocity = Velocity(Vec3 {
            x: 0.005,
            y: 0.005,
            z: 0.005,
        });
        let mat_comp2 = MaterialComponent(material.clone());
        let mesh_comp2 = MeshComponent(mesh.clone());
        let transform2 = Transform::new(
            Vec3::new(1.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::new(1.0, 1.0, 1.0),
        );
        let velocity2 = Velocity(Vec3 {
            x: 0.005,
            y: 0.005,
            z: -0.005,
        });

        let mut window = ResMut::<GLFWWindowRes>::retrieve_no_local(&context.application.world)
            .expect("Coudln't get window");
        window.0.set_cursor_mode(window::glfw::CursorMode::Disabled);

        drop(gl);
        drop(window);

        context
            .application
            .world
            .spawn_entity((mesh_comp, mat_comp, transform, velocity)).1
            .spawn_entity((mesh_comp2, mat_comp2, transform2, velocity2)).1
            .spawn_entity((
                Camera::new_perspective_default(16.0 / 9.0),
                Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
                CameraMovementComponent {
                    speed: 0.02,
                    mouse_sensitivity: 0.01,
                },
            ))
            .1
            .add_system(movement::move_system)
            .add_system(move_camera_system)
            .add_system(quit_app);
    }

    fn uninit<'a, 'b>(&self, _context: engine::plugin::PluginContext<'a, 'b>) {}
}
