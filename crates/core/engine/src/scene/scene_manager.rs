use crate::scene::scene::Scene;



pub struct SceneManager {
    current_scene: Scene,
}

impl SceneManager {
    pub fn new()  -> Self{
        let current_scene = Scene::new();
        SceneManager { current_scene }
    }

    pub fn update(self: &mut Self) -> () {
        self.current_scene.update();
    }
}
