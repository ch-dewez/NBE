pub mod shaders {
    include!(concat!(env!("OUT_DIR"), "/generated_shaders.rs"));
}

pub extern crate vulkanalia;

pub mod buffer;
pub mod commands;
mod config;
pub mod context;
mod framebuffer;
mod logical_device;
mod physical_device;
pub mod pipeline;
mod queue_family;
mod renderpass;
mod swapchains;
mod sync_objects;
pub mod texture;
