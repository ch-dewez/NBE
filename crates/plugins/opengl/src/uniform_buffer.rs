use core::slice;
use std::{cmp::max, sync::OnceLock };

use glow::{HasContext, NativeBuffer};

use crate::context::OpenGlContext;

pub struct UniformBuffer {
    buffer: glow::Buffer,
    bind_index: u32,
    usage : u32
}

impl UniformBuffer {
    pub fn new<T>(gl: &OpenGlContext, bind_index:u32, usage: u32) -> Self {
        unsafe {
            let buffer = gl.0.create_buffer().expect("Couldn't create buffer");
            gl.0.bind_buffer(glow::UNIFORM_BUFFER, Some(buffer));

            gl.0.buffer_data_size(glow::UNIFORM_BUFFER, size_of::<T>() as i32, usage);

            Self {
                buffer,
                bind_index,
                usage
            }
        }
    }

    pub fn update_data<T>(&self, gl: &OpenGlContext, data: &T){

        let slice = std::slice::from_ref(data);
        let ptr = slice.as_ptr() as *const u8; 
        let byte_buffer = unsafe {slice::from_raw_parts(ptr, size_of::<T>())};

        unsafe {
            gl.0.bind_buffer(glow::UNIFORM_BUFFER, Some(self.buffer));
            gl.0.buffer_data_u8_slice(glow::UNIFORM_BUFFER, byte_buffer, self.usage)
        }
    }

    pub fn bind(&self, gl: &OpenGlContext){
        unsafe {
            gl.0.bind_buffer_base(glow::UNIFORM_BUFFER, self.bind_index, Some(self.buffer));
        }
    }
}

pub struct GrowableUniformBuffer {
    buffer: NativeBuffer,

    bind_index: u32,

    pub len: usize, // in elements
    pub capacity: usize, // in elements
    size_of_element: usize,

    free_ids: Vec<usize>
}

static UNIFORM_ALIGNMENT: OnceLock<usize> =  OnceLock::new();

fn calculate_aligned_size(gl: &OpenGlContext, size:usize) -> usize {
    let alignment = UNIFORM_ALIGNMENT.get_or_init(||{
        unsafe{
            gl.0.get_parameter_i32(glow::UNIFORM_BUFFER_OFFSET_ALIGNMENT) as usize
        }
    });

    size + (alignment - (size % alignment))

}

impl GrowableUniformBuffer{
    pub fn new(gl: &OpenGlContext, bind_index:u32, capacity: usize, size_of_element: usize) -> Self{
        let size = calculate_aligned_size(gl, size_of_element);

        unsafe {
            let buffer = gl.0.create_buffer().expect("Couldn't create buffer");
            gl.0.bind_buffer(glow::UNIFORM_BUFFER, Some(buffer));

            gl.0.buffer_data_size(glow::UNIFORM_BUFFER, (capacity * size) as i32, glow::DYNAMIC_DRAW);

            Self {
                buffer,
                bind_index,

                len: 0,
                capacity,
                size_of_element: size,

                free_ids: Default::default()
            }
        }
    }

    pub fn grow(&mut self, gl: &OpenGlContext, new_capacity: usize){
        let new_capacity = max(self.capacity * 2, new_capacity);
        unsafe {
            // Create new buffer
            let buffer = gl.0.create_buffer().expect("Couldn't create buffer");
            // bind new buffer with write
            gl.0.bind_buffer(glow::COPY_WRITE_BUFFER, Some(buffer));
            //  bind old buffer with read
            gl.0.bind_buffer(glow::COPY_READ_BUFFER, Some(self.buffer));

            // allocate new buffer
            gl.0.buffer_data_size(glow::COPY_WRITE_BUFFER, (new_capacity * self.size_of_element) as i32, glow::DYNAMIC_COPY);

            // copy data
            gl.0.copy_buffer_sub_data(glow::COPY_READ_BUFFER, glow::COPY_WRITE_BUFFER, 0, 0, (self.capacity * self.size_of_element) as i32);

            gl.0.delete_buffer(self.buffer);
            self.buffer = buffer;
        }
        self.capacity = new_capacity;
    }

    pub fn get_id(&mut self, gl: &OpenGlContext) -> usize{
        if let Some(id) = self.free_ids.pop(){
            return id
        }

        if self.len < self.capacity {
            let id = self.len;
            self.len += 1;
            return id
        }

        self.grow(gl, self.capacity * 2);
        self.get_id(gl)
    }

    pub fn update_data<T>(&mut self, gl: &OpenGlContext, id:usize, data:&T){

        let offset = id * self.size_of_element;
        let slice = std::slice::from_ref(data);

        let ptr = slice.as_ptr() as *const u8; 
        let byte_buffer = unsafe {slice::from_raw_parts(ptr, self.size_of_element)};

        unsafe {
            gl.0.bind_buffer(glow::UNIFORM_BUFFER, Some(self.buffer));

            gl.0.buffer_sub_data_u8_slice(glow::UNIFORM_BUFFER, offset as i32, byte_buffer);
        }
    }

    pub fn bind(&self, gl:&OpenGlContext, id: usize){
        let offset = id * self.size_of_element;
        unsafe {
            gl.0.bind_buffer_range(glow::UNIFORM_BUFFER, self.bind_index, Some(self.buffer), offset as i32, self.size_of_element as i32);
        }

    }
}
