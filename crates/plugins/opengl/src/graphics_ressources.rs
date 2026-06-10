use std::{collections::HashMap, rc::Rc};

use ecs::ressource::Ressource;
use glow::Program;

use crate::{material::{Material, MaterialTemplate}, mesh::Mesh};

#[derive(Clone)]
pub enum GraphicsRessource {
    Mesh(Rc<Mesh>),
    Program(Rc<Program>),
    Material(Rc<Material>),
    MaterialTemplate(Rc<MaterialTemplate>),
}

#[derive(Default)]
pub struct GraphicsRessourceManager {
    ressources: HashMap<&'static str, GraphicsRessource>
}

impl GraphicsRessourceManager {
    pub fn get_or_add(&mut self, key: &'static str, ressource: GraphicsRessource) -> GraphicsRessource {
        self
            .ressources
            .entry(key)
            .or_insert(ressource)
            .clone()
    }

    pub fn get(&mut self, key: &'static str) -> Option<GraphicsRessource> {
        self
            .ressources
            .get(key)
            .cloned()
    }

    pub fn add(&mut self, key: &'static str, ressource: GraphicsRessource){
        self
            .ressources
            .insert(key, ressource);
    }

    pub fn remove(&mut self, key: &'static str){
        self
            .ressources
            .remove(key);
    }

    pub fn clear(&mut self){
        self.ressources.clear();
    }
} 

impl Ressource for GraphicsRessourceManager {}

