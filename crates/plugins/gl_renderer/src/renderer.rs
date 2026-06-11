use core_components::transform::Transform;
use ecs::{query::Query, ressource::{Res, ResMut, Ressource}, system::SystemParam};
use engine::plugin::Plugin;
use glam::Mat4;
use opengl::{CAMERA_BIND_INDEX, DEFAULT_UNIFORM_CAPACITY, MODEL_BIND_INDEX, context::OpenGlContext, glow::{self, HasContext}, material::MaterialComponent, mesh::MeshComponent, uniform_buffer::{GrowableUniformBuffer, UniformBuffer}};
use window::{glfw::{Context}, window::{GLFWRes, GLFWWindowRes}};

use crate::camera::{Camera, project_update_on_resize};


#[derive(Default)]
pub struct GlRenderer {}

fn clear(gl: Res<OpenGlContext>){
    unsafe {
        gl.0.clear_color(0.2, 0.3, 0.3, 1.0);
        gl.0.clear(opengl::glow::COLOR_BUFFER_BIT | opengl::glow::DEPTH_BUFFER_BIT);
    }
}

fn camera_ubo_update(query: Query<(&Camera, &Transform)>, camera_ubo: Res<CameraUBO>, gl: Res<OpenGlContext>){
    for (camera, transform) in query.into_iter(){
        let inv_rotation = glam::Quat::inverse(transform.rotation);
        let matrix = glam::Mat4::from_quat(inv_rotation);

        let translation = glam::Mat4::from_translation(-transform.position);

        let view_matrix = matrix * translation;

        let ubo_data = CameraUboData {
            view: view_matrix,
            proj: camera.projection
        };

        camera_ubo.0.update_data(&gl, &ubo_data);
    }
}


fn render_system(query: Query<(&Transform, &MeshComponent, &MaterialComponent)>,gl: Res<OpenGlContext>, mut model_ubo: ResMut<ModelUBO>){
    let model_count = query.count();
    if model_ubo.0.capacity < model_count{
        model_ubo.0.grow(&gl, model_count);
    }

    for (index, (transform, mesh, material)) in query.into_iter().enumerate(){
        model_ubo.0.update_data(&gl, index, &transform.get_rotation_matrix());
        model_ubo.0.bind(&gl, index);

        material.0.bind(&gl);

        mesh.0.bind(&gl);
        mesh.0.draw(&gl);
    }
}

fn swap_buffers(mut window: ResMut<GLFWWindowRes>){
    window.0.swap_buffers();
}

struct ModelUBO(GrowableUniformBuffer);
impl Ressource for ModelUBO{}

#[allow(dead_code)]
struct CameraUboData{
    view: Mat4,
    proj: Mat4
}
struct CameraUBO(UniformBuffer);
impl Ressource for CameraUBO{}

impl Plugin for GlRenderer {
    fn init<'a, 'b>(&self, context:engine::plugin::PluginContext<'a, 'b>) {
        let gl = Res::<OpenGlContext>::retrieve_no_local(&context.application.world).expect("Couldn't get open gl context");

        unsafe {
            gl.0.enable(opengl::glow::DEPTH_TEST);

            gl.0.enable(opengl::glow::CULL_FACE);

            gl.0.cull_face(opengl::glow::BACK);
        }

        let model_ubo = GrowableUniformBuffer::new(&gl, MODEL_BIND_INDEX, DEFAULT_UNIFORM_CAPACITY, size_of::<Mat4>());
        let camera_ubo = UniformBuffer::new::<CameraUboData>(&gl, CAMERA_BIND_INDEX, glow::DYNAMIC_DRAW);
        camera_ubo.bind(&gl);

        drop(gl);

        let mut glfw = ResMut::<GLFWRes>::retrieve_no_local(&context.application.world).expect("Can't get GLFW");
        glfw.0.set_swap_interval(window::glfw::SwapInterval::None);

        drop(glfw);

        context.application.world
            .add_system(clear)
            .add_system(camera_ubo_update)
            .add_system(render_system)
            .add_system(swap_buffers)
            .add_system(project_update_on_resize)
            .add_ressource(ModelUBO(model_ubo))
            .add_ressource(CameraUBO(camera_ubo));
    }

    fn uninit<'a, 'b>(&self, _context:engine::plugin::PluginContext<'a, 'b>) {
        
    }
}

