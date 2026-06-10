use core::slice;
use std::rc::Rc;

use ecs::component::Component;
use glam::Vec3;
use glow::{HasContext, NativeBuffer, NativeVertexArray};

use crate::context::OpenGlContext;
#[derive(PartialEq, Eq)]
enum MeshStoredLocation {
    Gpu,
    Cpu,
    Both
}

pub struct Mesh {
    nb_indices: u32,

    vertex_data: Option<Box<[Vec3]>>,
    index_data: Option<Box<[u32]>>,
    
    stored_location: MeshStoredLocation,

    vbo: Option<NativeBuffer>,
    vao: Option<NativeVertexArray>,
    ebo: Option<NativeBuffer>,
}

impl Mesh {
    pub fn new(vertex: Box<[Vec3]>, index: Box<[u32]>) -> Self{
        Self{
            nb_indices: index.len() as u32,

            vertex_data: Some(vertex),
            index_data: Some(index),

            stored_location: MeshStoredLocation::Cpu,

            vbo: None,
            vao: None,
            ebo: None,
        }
    }

    pub fn delete_from_cpu(&mut self){
        assert!(self.stored_location == MeshStoredLocation::Both);

        self.vertex_data = None;
        self.index_data = None;
        self.stored_location = MeshStoredLocation::Gpu;
    }

    pub fn copy_to_gpu(&mut self, gl: &OpenGlContext){
        assert!(self.stored_location == MeshStoredLocation::Cpu);

        // BIND VAO
        let vao = unsafe {gl.0.create_vertex_array()}. expect("Couldn't create vao");
        unsafe { gl.0.bind_vertex_array(Some(vao)) };

        {
            // BIND VBO
            let vbo = unsafe {gl.0.create_buffer() }.expect("Couldn't create vbo");
            unsafe { gl.0.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)) } ; 

            let slice = &**(self
                .vertex_data
                .as_ref()
                .expect("Copy to gpu but not stored on cpu"));
            let ptr = slice.as_ptr() as *const u8; 
            let byte_len = slice.len() * core::mem::size_of::<Vec3>();

            let byte_buffer = unsafe {slice::from_raw_parts(ptr, byte_len)};

            unsafe { gl.0.buffer_data_u8_slice(glow::ARRAY_BUFFER, byte_buffer, glow::STATIC_DRAW) };

            self.vbo = Some(vbo);
        }

        {
            // Vertex Attributes
            // this gl implement will force the mesh data to be at 0 location
            unsafe {
                gl.0.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
                gl.0.enable_vertex_attrib_array(0);
            };
        }

        {
            // BIND EBO
            let ebo = unsafe {gl.0.create_buffer() }.expect("Couldn't create ebo");
            unsafe { gl.0.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo)) } ; 

            let slice = &**(self
                .index_data
                .as_ref()
                .expect("Copy to gpu but not stored on cpu"));
            let ptr = slice.as_ptr() as *const u8; 
            let byte_len = slice.len() * core::mem::size_of::<u32>();

            let byte_buffer = unsafe {slice::from_raw_parts(ptr, byte_len)};

            unsafe { gl.0.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, byte_buffer, glow::STATIC_DRAW) };

            self.ebo = Some(ebo);
        }

        self.vao = Some(vao);

        unsafe { gl.0.bind_vertex_array(None) };

        self.stored_location = MeshStoredLocation::Both;
    }

    pub fn move_to_gpu(&mut self, gl: &OpenGlContext){
        self.copy_to_gpu(gl);
        self.delete_from_cpu();
    }

    pub fn bind(&self, gl: &OpenGlContext){
        assert!(self.stored_location != MeshStoredLocation::Cpu);

        unsafe {
            gl.0.bind_vertex_array(self.vao);
        }
    }

    pub fn draw(&self, gl: &OpenGlContext){
        unsafe { gl.0.draw_elements(glow::TRIANGLES, self.nb_indices as i32, glow::UNSIGNED_INT, 0);}
    }
}

pub struct MeshComponent (pub Rc<Mesh>);
impl Component for MeshComponent {}
