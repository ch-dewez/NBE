pub mod renderer;
pub mod camera;

pub mod default_shaders {
    include!(concat!(env!("OUT_DIR"), "/generated_shaders.rs"));
}
