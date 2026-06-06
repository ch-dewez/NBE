use std::{collections::HashSet};

use crate::{ressource::{Res, ResMut, Ressource}, system::{SystemDependency, SystemParam}, system_local::{Local, LocalStorage, SystemLocal}, world::{FromWorld, World}};

pub trait Event: 'static {}

/// This ressource behind a SingleEvent
pub struct SingleEventRes<T: Event>{
    pub(crate) event: Option<T>
}
impl<T: Event> Ressource for SingleEventRes<T> {}
impl<T: Event> SingleEventRes<T>{
    pub(crate) fn new() -> Self {
        Self{
            event:None
        }
    }
}

pub struct EventRes<T: Event> {
    pub(crate) previous_frame: Vec<(usize, T)>,
    pub(crate) this_frame: Vec<(usize, T)>,

    next_id: usize
}
impl<T: Event> Ressource for EventRes<T> {}
impl<T: Event> EventRes<T>{
    pub(crate) fn new() -> Self {
        Self{
            this_frame:Default::default(),
            previous_frame:Default::default(),
            next_id: 0
        }
    }

    pub(crate) fn swap(&mut self){
        std::mem::swap(&mut self.previous_frame, &mut self.this_frame);
    }
    pub(crate) fn clear_current(&mut self){
        self.this_frame.clear();
    }
}

/// Single Event Writer
/// Works the same way as an event, but it can only stores one event
/// there should only be one event writer in the systems but having multiple doesn't break anything
/// except that some events might not get handled properly
pub struct SingleEventWriter<'a, T: Event>(ResMut<'a, SingleEventRes<T>>);
/// Single Event Reader 
/// Works the same way as an event, but it can only stores one event
/// there should only be one event writer in the systems but having multiple doesn't break anything
/// except that some events might not get handled properly
pub struct SingleEventReader<'a, T: Event>(Res<'a, SingleEventRes<T>>);

/// Event writer, use to dispatch/emit an event. Allow for multiple event in a single frame
pub struct EventWriter<'a, T: Event>(ResMut<'a, EventRes<T>>);
/// Event reader, use to receive an event. Allow for multiple event in a single frame, it has 
/// a iter() function so you should iterate over the events and handle each of them
pub struct EventReader<'w, 'l, T: Event>{ 
    res:Res<'w, EventRes<T>>,
    local: Local<'l, EventReaderNextEvent>
}
struct EventReaderNextEvent(usize);
impl SystemLocal for EventReaderNextEvent{}
impl FromWorld for EventReaderNextEvent{
    fn from_world(_world: &World) -> Self {
        EventReaderNextEvent(0)
    }
}

enum EventReaderIteratorIndex{
    PreviousFrame(usize),
    ThisFrame(usize)
}
pub struct EventReaderIter<'w, 'l, T:Event>{
    pub(crate) reader: &'w EventReader<'w, 'l, T>,
    index: EventReaderIteratorIndex,
    next_index_to_visit: usize
}

macro_rules! impl_event_system_param {
    ($res:ident, $t:ident, $event_res:ident) => {

        impl<'a, T: Event> SystemParam for $t<'a, T> {
            type Item<'w, 'l> = $t<'w, T>;
            type Cache = ();

            fn init(world: &World, local:&mut LocalStorage){
                $res::<$event_res<T>>::init(world, local);
            }

            fn retrieve<'w, 'l>(world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>>{
                $res::<'w, $event_res<T>>::retrieve(world, local).map(move |res| $t(res))
            }

            fn cache(world: &World, local: &LocalStorage) -> Option<Self::Cache>{
                $res::<$event_res<T>>::cache(world, local)
            }

            fn from_cache<'w, 'l>(cache: &Self::Cache, world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>>{
                $res::<'w, $event_res<T>>::from_cache(cache, world, local).map(move |el |$t(el))
            }


            fn get_dependencies(world: &World, local: &LocalStorage) -> HashSet<SystemDependency>{
                $res::<$event_res<T>>::get_dependencies(world, local)
            }

            fn get_dependencies_from_cache(world: &World, cache: &Self::Cache, local: &LocalStorage) -> HashSet<SystemDependency>{
                $res::<$event_res<T>>::get_dependencies_from_cache(world, cache, local)

            }
        }
    };
}
impl_event_system_param!(Res, SingleEventReader, SingleEventRes);
impl_event_system_param!(ResMut, SingleEventWriter, SingleEventRes);
impl_event_system_param!(ResMut, EventWriter, EventRes);

impl<'a, 'b, T: Event> SystemParam for EventReader<'a, 'b, T> {
    type Item<'w, 'l> = EventReader<'w, 'l, T>;
    type Cache = ();

    fn init(world: &World, local:&mut LocalStorage){
        Res::<EventRes<T>>::init(world, local);
        Local::<EventReaderNextEvent>::init(world, local);
    }

    fn retrieve<'w, 'l>(world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>>{
        let res_opt = Res::<'w, EventRes<T>>::retrieve(world, local);
        let local_event_opt = Local::<'l, EventReaderNextEvent>::retrieve(world, local);

        if let (Some(res), Some(local_event)) = (res_opt, local_event_opt){
            Some(EventReader { res, local: local_event })
        }else{
            None
        }
    }

    fn cache(world: &World, local: &LocalStorage) -> Option<Self::Cache>{
        Res::<EventRes<T>>::cache(world, local)
    }

    fn from_cache<'w, 'l>(cache: &Self::Cache, world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>>{
        let res_opt = Res::<'w, EventRes<T>>::from_cache(cache, world, local);
        let local_event_opt = Local::<'l, EventReaderNextEvent>::from_cache(cache, world, local);

        if let (Some(res), Some(local_event)) = (res_opt, local_event_opt){
            Some(EventReader { res, local: local_event })
        }else{
            None
        }
    }


    fn get_dependencies(world: &World, local: &LocalStorage) -> HashSet<SystemDependency>{
        Res::<EventRes<T>>::get_dependencies(world, local)
    }

    fn get_dependencies_from_cache(world: &World, cache: &Self::Cache, local: &LocalStorage) -> HashSet<SystemDependency>{
        Res::<EventRes<T>>::get_dependencies_from_cache(world, cache, local)

    }
}

impl<'a, T: Event> SingleEventWriter<'a, T>{
    pub fn write(&mut self, value: T){
        self.0.event = Some(value);
    }
}

impl<'a, T: Event> EventWriter<'a, T>{
    pub fn write(&mut self, value: T){
        let id = self.0.next_id;
        self.0.next_id += 1;
        self.0.this_frame.push((id, value));
    }
}


impl<'a, T: Event> SingleEventReader<'a, T>{
    pub fn read(&self) -> Option<&T>{
        self.0.event.as_ref()
    }
}

impl<'a, 'b, T:Event> EventReader<'a, 'b, T>{
    pub fn iter<'w, 'l, 'r>(&'r mut self) -> EventReaderIter<'w, 'l, T>
    where 'r: 'w+'l{
        let next_index = self.local.0;
        self.local.0 = self.res.next_id;
        EventReaderIter { reader: self, index: EventReaderIteratorIndex::PreviousFrame(0), next_index_to_visit:next_index}
    }
}

impl<'a, 'b, T:Event> Iterator for EventReaderIter<'a, 'b, T>{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.index {
                EventReaderIteratorIndex::PreviousFrame(index) => {
                    let value = self.reader.res.previous_frame.get(index);
                    if let Some(event) = value{
                        if event.0 >= self.next_index_to_visit {
                            self.next_index_to_visit = event.0 + 1;
                            return Some(&event.1);
                        }else {
                            self.index = EventReaderIteratorIndex::PreviousFrame(index + 1);
                            continue;
                        }
                    }

                    self.index = EventReaderIteratorIndex::ThisFrame(0);
                }
                EventReaderIteratorIndex::ThisFrame(index) => {
                    let value = self.reader.res.this_frame.get(index);
                    if let Some(event) = value{
                        if event.0 >= self.next_index_to_visit {
                            self.next_index_to_visit = event.0 + 1;
                            return Some(&event.1);
                        }else {
                            self.index = EventReaderIteratorIndex::ThisFrame(index + 1);
                            continue;
                        }
                    }
                    return None;
                }
            }
        }
    }
}
