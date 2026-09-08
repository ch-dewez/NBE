use std::rc::Rc;

use core_components::transform::Transform;
use ecs::{
    command::Command,
    event::EventReader,
    ressource::{Res, ResMut},
    system::SystemParam,
};
use engine::{app_command::StopAppCommand, application::DeltaTimeS, plugin::Plugin};
use gl_renderer::{camera::Camera, default_shaders::DefaultShaders};
use glam::{Quat, Vec2, Vec3, Vec4};
use opengl::{
    context::OpenGlContext,
    material::{Material, MaterialComponent, MaterialTemplate},
    mesh::{Mesh, MeshComponent},
    program::Program,
};
use physics::collision::colliders::{Colliders, CubeCollider};
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

fn fps_printer(dt: Res<DeltaTimeS>) {
    println!("FPS: {}", 1.0 / dt.0);
}

#[derive(Default)]
pub struct GamePlugin {}

impl Plugin for GamePlugin {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {
        // The type is now Box<[(Vec3, Vec3)]> where:
        // - The first Vec3 is Position (X, Y, Z)
        // - The second Vec3 is Color (R, G, B)
        let vertices: Box<[Vec3]> = Box::new([
            // Front face (z = 0.5) - Reddish tones
            (Vec3::new(-0.5, -0.5, 0.5)), // 0: Bottom-left-front (Red)
            (Vec3::new(0.5, -0.5, 0.5)),  // 1: Bottom-right-front (Orange)
            (Vec3::new(0.5, 0.5, 0.5)),   // 2: Top-right-front (Yellow)
            (Vec3::new(-0.5, 0.5, 0.5)),  // 3: Top-left-front (Magenta)
            // Back face (z = -0.5) - Bluish/Greenish tones
            (Vec3::new(-0.5, -0.5, -0.5)), // 4: Bottom-left-back (Blue)
            (Vec3::new(0.5, -0.5, -0.5)),  // 5: Bottom-right-back (Green)
            (Vec3::new(0.5, 0.5, -0.5)),   // 6: Top-right-back (Cyan)
            (Vec3::new(-0.5, 0.5, -0.5)),  // 7: Top-left-back (White)
        ]);

        let _vertices_texture: Box<[(Vec3, Vec3, Vec2)]> = Box::new([
            // Front face (z = 0.5) - Reddish tones
            (
                Vec3::new(-0.5, -0.5, 0.5),
                Vec3::new(1.0, 0.0, 0.0),
                Vec2::new(0.0, 0.0),
            ),
            (
                Vec3::new(0.5, -0.5, 0.5),
                Vec3::new(1.0, 0.5, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (
                Vec3::new(0.5, 0.5, 0.5),
                Vec3::new(1.0, 1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ),
            (
                Vec3::new(-0.5, 0.5, 0.5),
                Vec3::new(1.0, 0.0, 1.0),
                Vec2::new(0.0, 1.0),
            ),
            // Back face (z = -0.5) - Bluish/Greenish tones
            (
                Vec3::new(-0.5, -0.5, -0.5),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(1.0, 0.0),
            ),
            (
                Vec3::new(0.5, -0.5, -0.5),
                Vec3::new(0.0, 1.0, 0.0),
                Vec2::new(0.0, 0.0),
            ),
            (
                Vec3::new(0.5, 0.5, -0.5),
                Vec3::new(0.0, 1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ),
            (
                Vec3::new(-0.5, 0.5, -0.5),
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
            // Top face (y = 0.5)
            (
                Vec3::new(-0.5, 0.5, -0.5),
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(0.0, 0.0),
            ),
            (
                Vec3::new(0.5, 0.5, -0.5),
                Vec3::new(0.0, 1.0, 1.0),
                Vec2::new(1.0, 0.0),
            ),
            (
                Vec3::new(0.5, 0.5, 0.5),
                Vec3::new(1.0, 1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ),
            (
                Vec3::new(-0.5, 0.5, 0.5),
                Vec3::new(1.0, 0.0, 1.0),
                Vec2::new(0.0, 1.0),
            ),
            // Bottom face (y = -0.5)
            (
                Vec3::new(-0.5, -0.5, -0.5),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(0.0, 1.0),
            ),
            (
                Vec3::new(0.5, -0.5, -0.5),
                Vec3::new(0.0, 1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ),
            (
                Vec3::new(0.5, -0.5, 0.5),
                Vec3::new(1.0, 0.5, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (
                Vec3::new(-0.5, -0.5, 0.5),
                Vec3::new(1.0, 0.0, 0.0),
                Vec2::new(0.0, 0.0),
            ),
            // Right face (x = 0.5)
            (
                Vec3::new(0.5, -0.5, 0.5),
                Vec3::new(1.0, 0.5, 0.0),
                Vec2::new(0.0, 0.0),
            ),
            (
                Vec3::new(0.5, -0.5, -0.5),
                Vec3::new(0.0, 1.0, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (
                Vec3::new(0.5, 0.5, -0.5),
                Vec3::new(0.0, 1.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
            (
                Vec3::new(0.5, 0.5, 0.5),
                Vec3::new(1.0, 1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ),
            // Left face (x = -0.5)
            (
                Vec3::new(-0.5, -0.5, -0.5),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(0.0, 0.0),
            ),
            (
                Vec3::new(-0.5, -0.5, 0.5),
                Vec3::new(1.0, 0.0, 0.0),
                Vec2::new(1.0, 0.0),
            ),
            (
                Vec3::new(-0.5, 0.5, 0.5),
                Vec3::new(1.0, 0.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
            (
                Vec3::new(-0.5, 0.5, -0.5),
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ),
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
        let _indices_dup: Box<[u32]> = Box::new([
            0, 1, 2, 0, 2, 3, // Front
            4, 6, 5, 4, 7, 6, // Back
            8, 10, 9, 8, 11, 10, // Top
            12, 13, 14, 12, 14, 15, // Bottom
            16, 17, 18, 16, 18, 19, // Right
            20, 21, 22, 20, 22, 23, // Left
        ]);

        {
            let mut window = ResMut::<GLFWWindowRes>::retrieve_no_local(&context.application.world)
                .expect("Coudln't get window");
            window.0.set_cursor_mode(window::glfw::CursorMode::Disabled);
        }

        let gl = Res::<OpenGlContext>::retrieve_no_local(&context.application.world)
            .expect("Couldn't get open gl context");

        // TODO: maybe I shoudl take a Rc<[]> to not clone
        let mut mesh = Mesh::new(vertices, indices.clone());
        mesh.move_to_gpu::<Vec3>(&gl);
        let mesh = Rc::new(mesh);

        let simple_color_template = {
            let simple_color_program = Rc::new(Program::new(
                DefaultShaders::DefaultVert.code(),
                DefaultShaders::DefaultFrag.code(),
                &gl,
            ));
            Rc::new(MaterialTemplate::new_no_textures::<Vec4>(
                &gl,
                simple_color_program,
            ))
        };

        let light_color = Vec4::new(0.98, 0.89, 0.69, 1.0);
        let light_color_material = Rc::new(Material::new_no_textures(
            &gl,
            simple_color_template.clone(),
        ));
        light_color_material.update_data(&gl, &light_color);
        let dark_color = Vec4::new(0.29, 0.23, 0.13, 1.0);
        let dark_color_material = Rc::new(Material::new_no_textures(
            &gl,
            simple_color_template.clone(),
        ));
        dark_color_material.update_data(&gl, &dark_color);
        drop(gl);

        let world = &mut context.application.world;

        {
            // GROUND
            let mat_comp = MaterialComponent(light_color_material.clone());
            let mesh_comp = MeshComponent(mesh.clone());
            let transform = Transform::new(
                Vec3::new(0.0, -1.0, -2.0),
                Quat::IDENTITY,
                Vec3::new(10.0, 1.0, 10.0),
            );

            let collider = CubeCollider::default();
            let colliders = Colliders::from(collider);

            world.spawn_entity((transform, mesh_comp, mat_comp, colliders));
        }

        {
            // CUBE
            let mat_comp = MaterialComponent(dark_color_material.clone());

            let mesh_comp = MeshComponent(mesh.clone());
            let transform = Transform::new(
                Vec3::new(0.0, 1.0, -2.0),
                Quat::from_xyzw(0.88807, 0.32506, -0.32506, 0.0), // cube standing (more or less) on its vertex
                Vec3::new(1.0, 1.0, 1.0),
            );

            let move_comp = Velocity::from(Vec3::new(0.0, -1.0, 0.0));

            let collider = CubeCollider::default();
            let colliders = Colliders::from(collider);

            world.spawn_entity((transform, mesh_comp, mat_comp, move_comp, colliders));
        }

        {
            let transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::new(1.0, 1.0, 1.0));
            let camera = Camera::new_perspective_default(16.0 / 9.0);
            let camera_movement = CameraMovementComponent {
                speed: 2.0,
                mouse_sensitivity: 0.01,
            };

            let collider = CubeCollider::default();
            let colliders = Colliders::from(collider);

            world.spawn_entity((transform, camera, camera_movement, colliders));
        }

        world
            .add_system(movement::move_system)
            .add_system(move_camera_system)
            .add_system(fps_printer)
            .add_system(quit_app);
    }

    fn uninit<'a, 'b>(&self, _context: engine::plugin::PluginContext<'a, 'b>) {}
}
