use std::{default, marker::PhantomData, sync::Arc};

use ecs::{
    ressource::{Res, ResMut},
    system::SystemParam,
};
use engine::plugin::Plugin;
use logger::log::error;
use rhi_lib::traits::{RHIContextAllTraits, RHIWindowInfo};
use window::{
    glfw::ClientApiHint::NoApi,
    window::{GLFWRes, GLFWWindowRes, WindowPlugin},
};

use crate::ressource::RHIContextRes;

#[derive(Debug)]
pub struct RHIPlugin<C: RHIContextAllTraits> {
    phantom_data: PhantomData<C>,
}

impl<C: RHIContextAllTraits> Default for RHIPlugin<C> {
    fn default() -> Self {
        Self {
            phantom_data: Default::default(),
        }
    }
}

impl<C: RHIContextAllTraits> Plugin for RHIPlugin<C> {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {
        // TODO: Not depend on the glfw window plugin. But I'm not sure how I can implement hint without this
        let mut glfw = ResMut::<GLFWRes>::retrieve_no_local(&context.application.world)
            .expect("GLFW window not loaded, cannot load glow");

        // TODO: Move this to the context, hints might be different for different backend
        glfw.0
            .window_hint(window::glfw::WindowHint::ClientApi(NoApi));

        drop(glfw);

        WindowPlugin::create_window(&mut context.application.world);

        let window = Res::<GLFWWindowRes>::retrieve_no_local(&context.application.world)
            .expect("Couldn't get glfw window handle");

        let window_res = window.0.get_size();

        let graphic_context = C::create(RHIWindowInfo {
            window: &window.0,
            display: &window.0,
            window_res: (window_res.0 as u32, window_res.1 as u32),
        })
        .map_err(|err| error!("Couldn't create graphic context : {}", err))
        .unwrap();
        drop(window);
        let rhi_context = RHIContextRes(Arc::new(graphic_context));
        // let rhi_context_ref = RHIContextDynRes(rhi_context.0.clone());

        context.application.world.add_ressource(rhi_context);
        // .add_ressource(rhi_context_ref);
    }

    fn uninit<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {}
}
