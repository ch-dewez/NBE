use engine::plugin::Plugin;

use crate::collision::collision_management::manage_collision;

#[derive(Default)]
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {
        println!("Adding system");
        context.application.world.add_system(manage_collision);
    }
    fn uninit<'a, 'b>(&self, _context: engine::plugin::PluginContext<'a, 'b>) {}
}
