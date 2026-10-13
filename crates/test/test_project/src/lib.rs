mod camera_movement;
mod movement;

mod assets {
    include!(concat!(env!("OUT_DIR"), "/generated_assets.rs"));
}

pub mod plugin;
