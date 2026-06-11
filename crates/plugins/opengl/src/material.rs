use std::{cell::RefCell, rc::Rc};

use ecs::component::Component;

use crate::{DEFAULT_UNIFORM_CAPACITY, MATERIAL_BIND_INDEX, context::OpenGlContext, program::Program, texture::Texture, uniform_buffer::GrowableUniformBuffer};

pub struct MaterialTemplate {
    program: Rc<Program>,
    ubo: RefCell<GrowableUniformBuffer>,
}

impl MaterialTemplate {
    pub fn new_no_textures<T>(gl: &OpenGlContext, program: Rc<Program>) -> Self {
        let ubo = RefCell::new(GrowableUniformBuffer::new(gl, MATERIAL_BIND_INDEX, DEFAULT_UNIFORM_CAPACITY, size_of::<T>()));

        Self {
            program,
            ubo,
        }
    }

    pub fn new<T>(gl: &OpenGlContext, program: Rc<Program>, texture_names: &[&str]) -> Self{
        let ubo = RefCell::new(GrowableUniformBuffer::new(gl, MATERIAL_BIND_INDEX, DEFAULT_UNIFORM_CAPACITY, size_of::<T>()));

        let template = Self {
            program,
            ubo,
        };
        template.set_texture_locations(gl, texture_names);
        template
    }

    pub fn bind(&self, gl: &OpenGlContext, id: usize) {
        self.bind_program(gl);
        self.bind_ubo(gl, id);
    }

    pub fn bind_ubo(&self, gl: &OpenGlContext, id: usize) {
        self.ubo.borrow().bind(gl, id);
    }

    pub fn bind_program(&self, gl: &OpenGlContext) {
        self.program.bind(gl);
    }

    pub fn set_texture_locations(&self, gl: &OpenGlContext, texture_names: &[&str]){
        for (index, texture_name) in texture_names.iter().enumerate(){
            self.program.set_int(gl, texture_name, index as i32);
        }
    }
}

pub struct Material {
    template: Rc<MaterialTemplate>,
    textures: Box<[Rc<Texture>]>,
    id: usize
}

impl Material{
    pub fn new_no_textures(gl: &OpenGlContext, template: Rc<MaterialTemplate>) -> Self{
        let id = template.ubo.borrow_mut().get_id(gl);

        Self {template, id, textures: Default::default()}
    }

    pub fn new(gl: &OpenGlContext, template: Rc<MaterialTemplate>, textures: &[Rc<Texture>]) -> Self{
        let id = template.ubo.borrow_mut().get_id(gl);
        let textures: Box<[Rc<Texture>]> = Box::from(textures);

        Self {template, id, textures}
    }

    pub fn update_data<T>(&self, gl: &OpenGlContext, data: &T){
        self.template.ubo.borrow_mut().update_data(gl, self.id, data);
    }

    pub fn bind(&self, gl: &OpenGlContext) {
        self.bind_program(gl);
        self.bind_textures(gl);
        self.bind_data(gl);
    }

    pub fn bind_data(&self, gl: &OpenGlContext) {
        self.template.ubo.borrow().bind(gl, self.id);
    }

    pub fn bind_program(&self, gl: &OpenGlContext) {
        self.template.program.bind(gl);
    }

    pub fn bind_textures(&self, gl:&OpenGlContext){
        for (index, texture) in self.textures.iter().enumerate(){
            texture.bind(gl, index as u32);
        }
    }

}



pub struct MaterialComponent (pub Rc<Material>);
impl Component for MaterialComponent {}
