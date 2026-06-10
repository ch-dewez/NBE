use glow::{HasContext, NativeProgram};

use crate::{CAMERA_BIND_INDEX, CAMERA_BIND_NAME, MATERIAL_BIND_INDEX, MATERIAL_BIND_NAME, MODEL_BIND_INDEX, MODEL_BIND_NAME, context::OpenGlContext};


pub struct Program (pub NativeProgram);

impl Program {
    pub fn new(vertex_shader_source: &str, fragment_shader_source:&str, gl: &OpenGlContext) -> Self{
        let vertex_shader = unsafe {gl.0.create_shader(glow::VERTEX_SHADER) }.expect("Couldn't create vertex shader");

        unsafe { gl.0.shader_source(vertex_shader, vertex_shader_source) } ;
        unsafe { gl.0.compile_shader(vertex_shader) } ;
        unsafe {
            if ! gl.0.get_shader_compile_status(vertex_shader) {
                panic!( "{}", gl.0.get_shader_info_log(vertex_shader));
            }
        }


        let fragment_shader: glow::NativeShader = unsafe {gl.0.create_shader(glow::FRAGMENT_SHADER) }.expect("Couldn't create fragment shader");

        unsafe { gl.0.shader_source(fragment_shader, fragment_shader_source) } ;
        unsafe { gl.0.compile_shader(fragment_shader) } ;
        unsafe {
            if !gl.0.get_shader_compile_status(fragment_shader) {
                panic!( "{}", gl.0.get_shader_info_log(fragment_shader));
            }
        }

        let program = unsafe { gl.0.create_program()}.expect("Couldnt' create program");
        unsafe {
            gl.0.attach_shader(program, vertex_shader);
            gl.0.attach_shader(program, fragment_shader);

            gl.0.link_program(program);
            if !gl.0.get_program_link_status(program) {
            panic!( "{}", gl.0.get_program_info_log(program));
            }

            let block_index = gl.0.get_uniform_block_index(program, MODEL_BIND_NAME).expect("Couldn't find binding_name in the program(shader)");
            gl.0.uniform_block_binding(program, block_index, MODEL_BIND_INDEX);
            let block_index = gl.0.get_uniform_block_index(program, CAMERA_BIND_NAME).expect("Couldn't find binding_name in the program(shader)");
            gl.0.uniform_block_binding(program, block_index, CAMERA_BIND_INDEX);
            let block_index = gl.0.get_uniform_block_index(program, MATERIAL_BIND_NAME).expect("Couldn't find binding_name in the program(shader)");
            gl.0.uniform_block_binding(program, block_index, MATERIAL_BIND_INDEX);
                
            gl.0.delete_shader(vertex_shader);
            gl.0.delete_shader(fragment_shader);
        }


        Self (program)
    }

    pub fn bind(&self, gl: &OpenGlContext){
        unsafe { gl.0.use_program(Some(self.0)) } ;

    }
}
