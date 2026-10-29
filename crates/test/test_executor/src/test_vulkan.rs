use engine::plugin::Plugin;

#[derive(Debug, Default)]
pub struct TestVulkanPlugin {}

impl Plugin for TestVulkanPlugin {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {}

    fn uninit<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {}
}
