use derive_more::{Deref, DerefMut};
use ecs::ressource::Ressource;
use rhi_lib::traits::{RHIContext, RHIContextAllTraits};
use std::sync::Arc;

#[derive(Clone, Deref, DerefMut)]
pub struct RHIContextRes<C: RHIContextAllTraits>(pub Arc<C>);
impl<C: RHIContextAllTraits> Ressource for RHIContextRes<C> {}
// #[derive(Deref, DerefMut)]
// pub struct RHIContextDynRes(pub Arc<dyn RHIContext>);
// impl Ressource for RHIContextDynRes {}
