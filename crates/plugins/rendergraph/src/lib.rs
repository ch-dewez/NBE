use std::{marker::PhantomData, sync::Arc};

use derive_more::{Deref, DerefMut};
use ecs::{
    ressource::{Res, Ressource},
    system::SystemParam,
};
use engine::plugin::Plugin;
use rendergraph_lib::rendergraph::Rendergraph;
use rhi_lib::traits::RHIContextAllTraits;
use rhi_plugin::ressource::RHIContextRes;

pub struct RenderGraphPlugin<C: RHIContextAllTraits, const N: usize = 2>(PhantomData<C>);

impl<C: RHIContextAllTraits, const N: usize> Default for RenderGraphPlugin<C, N> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Deref, DerefMut)]
pub struct RenderGraphRes<C: RHIContextAllTraits, const N: usize>(pub Rendergraph<C, N>);
impl<C: RHIContextAllTraits, const N: usize> Ressource for RenderGraphRes<C, N> {}

impl<C: RHIContextAllTraits, const N: usize> Plugin for RenderGraphPlugin<C, N> {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {
        let rhi_context = Res::<RHIContextRes<C>>::retrieve_no_local(&context.application.world)
            .expect("Couldn't get Graphics backend context");

        let rendergraph: Rendergraph<C, N> = Rendergraph::new(Arc::clone(&rhi_context.0));
        let rendergraph_res = RenderGraphRes::<C, N>(rendergraph);

        drop(rhi_context);

        context.application.world.add_ressource(rendergraph_res);
    }

    fn uninit<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {}
}
