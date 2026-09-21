use engine::plugin::Plugin;

use crate::{
    collision::collision_management::manage_collision,
    rigidbody::{gravity_update, rb_update},
};

#[derive(Default)]
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {
        println!("Adding system");
        context
            .application
            .world
            .add_system(gravity_update)
            .add_system(manage_collision)
            .add_system(rb_update);
    }
    fn uninit<'a, 'b>(&self, _context: engine::plugin::PluginContext<'a, 'b>) {}
}
