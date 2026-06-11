use core::slice;
use std::rc::Rc;

use ecs::component::Component;
use glam::{Vec2, Vec3};
use glow::HasContext ;
use macro_utils::repeat_macro_with_argument_without_0;

use crate::context::OpenGlContext;
#[derive(PartialEq, Eq)]
enum MeshStoredLocation {
    Gpu,
    Cpu,
    Both
}

pub struct Mesh {
    nb_indices: u32,

    vertex_data: Option<Box<[u8]>>,
    index_data: Option<Box<[u32]>>,
    
    stored_location: MeshStoredLocation,

    vbo: Option<glow::Buffer>,
    vao: Option<glow::VertexArray>,
    ebo: Option<glow::Buffer>,
}

pub trait VertexAttribute{
    fn get_nb_component() -> i32;
    fn get_data_type() -> u32;
}

impl VertexAttribute for Vec3{
    fn get_data_type() -> u32 {
        glow::FLOAT
    }
    fn get_nb_component() -> i32 {
        3
    }
}
impl VertexAttribute for Vec2{
    fn get_data_type() -> u32 {
        glow::FLOAT
    }
    fn get_nb_component() -> i32 {
        2
    }
}
impl VertexAttribute for f32{
    fn get_data_type() -> u32 {
        glow::FLOAT
    }
    fn get_nb_component() -> i32 {
        1
    }
}
impl VertexAttribute for u32{
    fn get_data_type() -> u32 {
        glow::UNSIGNED_INT
    }
    fn get_nb_component() -> i32 {
        1
    }
}

pub trait VertexAttributeTupple {
    fn set_attribute(gl: &OpenGlContext);
}

macro_rules! impl_vertex_attribute_tupple {
    ($($params:ident),*) => {
        #[allow(unused_parens)]
        #[allow(unused_assignments)]
impl<$($params: VertexAttribute),*> VertexAttributeTupple for ($($params),*){
    fn set_attribute(gl: &OpenGlContext){
        let stride: i32 = (0 $(+ size_of::<$params>())*) as i32;
        let mut current_offset = 0;
        let mut current_index = 0;

        $(
        unsafe {
            gl.0.vertex_attrib_pointer_f32(current_index, $params::get_nb_component(), $params::get_data_type(), false, stride, current_offset);
            gl.0.enable_vertex_attrib_array(current_index);

            current_index += 1;
            current_offset += size_of::<$params>() as i32;
        }

        )*
    }
}
    };
}

// 12 seems enough
repeat_macro_with_argument_without_0!(impl_vertex_attribute_tupple, 12);

impl Mesh {
    pub fn new<T: VertexAttributeTupple>(vertex: Box<[T]>, index: Box<[u32]>) -> Self{
        let byte_len = size_of_val(vertex.as_ref());
        let slice = Box::into_raw(vertex) as *const u8;
        let vertex_box: Box<[u8]>;
        unsafe {
            let byte_buffer: &[u8] = slice::from_raw_parts(slice, byte_len);
            vertex_box = Box::from(byte_buffer);
        }

        Self{
            nb_indices: index.len() as u32,

            vertex_data: Some(vertex_box),
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

    pub fn copy_to_gpu<T: VertexAttributeTupple>(&mut self, gl: &OpenGlContext){
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
            let ptr = slice.as_ptr(); 
            let byte_len = size_of_val(slice);

            let byte_buffer = unsafe {slice::from_raw_parts(ptr, byte_len)};

            unsafe { gl.0.buffer_data_u8_slice(glow::ARRAY_BUFFER, byte_buffer, glow::STATIC_DRAW) };

            self.vbo = Some(vbo);
        }

        {
            // Vertex Attributes
            T::set_attribute(gl);
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
            let byte_len = size_of_val(slice);

            let byte_buffer = unsafe {slice::from_raw_parts(ptr, byte_len)};

            unsafe { gl.0.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, byte_buffer, glow::STATIC_DRAW) };

            self.ebo = Some(ebo);
        }

        self.vao = Some(vao);

        unsafe { gl.0.bind_vertex_array(None) };

        self.stored_location = MeshStoredLocation::Both;
    }

    pub fn move_to_gpu<T: VertexAttributeTupple>(&mut self, gl: &OpenGlContext){
        self.copy_to_gpu::<T>(gl);
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
