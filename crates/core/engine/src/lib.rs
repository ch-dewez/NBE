extern crate gl;

mod application;
pub use application::Application;
pub use application::ApplicationCreationInfo;

pub mod scene;

pub mod scripting;
pub use scripting::plugin::Plugin;

