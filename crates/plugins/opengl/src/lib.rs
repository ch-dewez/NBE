pub extern crate glow;

pub const DEFAULT_UNIFORM_CAPACITY: usize = 100;

pub const MODEL_BIND_NAME: &str = "Model";
pub const CAMERA_BIND_NAME: &str = "Camera";
pub const MATERIAL_BIND_NAME: &str = "Material";

pub const CAMERA_BIND_INDEX: u32 = 0;
pub const MODEL_BIND_INDEX: u32 = 1;
pub const MATERIAL_BIND_INDEX: u32 = 2;
pub const LIGHTNING_BIND_INDEX: u32 = 3;

pub mod context;
pub mod graphics_ressources;
pub mod material;
pub mod mesh;
pub mod program;
pub mod texture;
pub mod uniform_buffer;
