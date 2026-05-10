use engine::{Plugin, scene::component::ComponentTrait};

#[derive(Default)]
pub struct Game {
}

impl Plugin for Game {

}

struct DummyComponent {}
impl ComponentTrait for DummyComponent {}

// engine::create_system!(
//     hello_world,
//     DummyComponenet,
//     || {
//         println!("Hello_world");
//     }
// )


