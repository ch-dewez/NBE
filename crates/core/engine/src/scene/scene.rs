//
//
// pub struct Scene {
//     pub entities: [Option<Entity>;MAX_ENTITY_COUNT],
//
//     pub component_registry: ComponentRegistry,
//
//     pub systems: SystemPool
// }
//
// impl Scene {
//     pub fn new() -> Self {
//         Self {
//             entities: [None; MAX_ENTITY_COUNT],
//             component_registry: ComponentRegistry::default(),
//             systems: SystemPool::default()
//         }
//     }
//
//     pub fn add_entity(self: &mut Self, _entity: Entity) {
//         //self.entities.push(entity);
//     }
//
//     pub fn update(self: &mut Self) -> () {
//
//     }
// }
