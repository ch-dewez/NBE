use std::{cell::RefCell, rc::Rc};

use ecs::component::Component;

use crate::{DEFAULT_UNIFORM_CAPACITY, MATERIAL_BIND_INDEX, context::OpenGlContext, program::Program, uniform_buffer::GrowableUniformBuffer};

pub struct MaterialTemplate {
    program: Rc<Program>,
    ubo: RefCell<GrowableUniformBuffer>,
}

impl MaterialTemplate {
    pub fn new(gl: &OpenGlContext, program: Rc<Program>, size_of_element: usize) -> Self {
        let ubo = RefCell::new(GrowableUniformBuffer::new(gl, MATERIAL_BIND_INDEX, DEFAULT_UNIFORM_CAPACITY, size_of_element));

        Self {
            program,
            ubo
        }
    }
}

pub struct Material {
    template: Rc<MaterialTemplate>,
    id: usize
}

impl Material{
    pub fn new(template: Rc<MaterialTemplate>, gl: &OpenGlContext) -> Self{
        let id = template.ubo.borrow_mut().get_id(gl);

        Self {template, id}
    }

    pub fn update_data<T>(&self, gl: &OpenGlContext, data: &T){
        self.template.ubo.borrow_mut().update_data(gl, self.id, data);
    }

    pub fn bind(&self, gl: &OpenGlContext) {
        self.bind_program(gl);
        self.bind_ubo(gl);
    }

    pub fn bind_ubo(&self, gl: &OpenGlContext) {
        self.template.ubo.borrow().bind(gl, self.id);
    }

    pub fn bind_program(&self, gl: &OpenGlContext) {
        self.template.program.bind(gl);
    }

}



pub struct MaterialComponent (pub Rc<Material>);
impl Component for MaterialComponent {}
