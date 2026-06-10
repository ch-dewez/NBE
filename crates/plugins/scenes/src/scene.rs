use std::hint::unreachable_unchecked;

use ecs::world::World;

pub type SceneLoadFunction = Box<dyn Fn(&mut Scene)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EngineSceneId(pub usize);
impl From<usize> for EngineSceneId {
    fn from(id: usize) -> Self{
        EngineSceneId(id)
    }
}

pub struct SceneManager<'a> {
    scenes: Vec<(ManagedScene<'a>, SceneLoadFunction)>,
    active_scene: Option<EngineSceneId>,
    // global_scene: EngineSceneId // TODO: implement this, it would be a scene that never unload during
    // the app lifetime, useful for sounds or some state manager
}

pub enum SceneSteppingError{
    NoActiveScene,
    ActiveSceneUnloaded
}
pub enum RegisterSceneError {
    SceneIdAlreadyExist,
    SceneIdLeaveGap,
}
impl<'a> SceneManager<'a> {
    pub fn new() -> Self {
        SceneManager {
            scenes: vec![],
            active_scene: None
        }
    }

    pub fn register_scene(&mut self, id: EngineSceneId,  load_function: SceneLoadFunction) -> Result<(), RegisterSceneError>{
        if self.scenes.len() < id.0 {
            return Err(RegisterSceneError::SceneIdAlreadyExist);
        }else if self.scenes.len() > id.0 {
            return Err(RegisterSceneError::SceneIdLeaveGap);
        }
        self.scenes.push((ManagedScene::Unloaded, load_function));
        Ok(())
    }

    pub fn load_scene(&mut self, scene_id:EngineSceneId){
        let (scene, func) = &mut self.scenes[scene_id.0];

        *scene = ManagedScene::Loaded(Scene::default());

        match scene {
            ManagedScene::Loaded(scene) => func(scene),
            _ => unsafe {unreachable_unchecked()}
        }
    }

    pub fn unload_scene(&mut self, scene_id: EngineSceneId ){
        self.scenes[scene_id.0].0 = ManagedScene::Unloaded;
    }

    pub fn switch_scene(&mut self, scene_id: EngineSceneId){
        if let Some(id) = self.active_scene{
            self.unload_scene(id);
        }
            self.active_scene = Some(scene_id);
    }

    pub fn switch_scene_wihtout_unloading(&mut self, scene_id:EngineSceneId){
        self.active_scene = Some(scene_id);

    }

    pub fn step(&mut self) -> Result<(), SceneSteppingError>{
        match self.active_scene {
            Some(id ) =>  {
                match &mut self.scenes[id.0].0 {
                    ManagedScene::Loaded(scene) => scene.step(),
                    ManagedScene::Unloaded => Err(SceneSteppingError::ActiveSceneUnloaded)?
                };
            }
            None => Err(SceneSteppingError::NoActiveScene)?,
        };
        Ok(())
    }
}

impl<'a> Default for SceneManager<'a> {
    fn default() -> Self {
        SceneManager::new()
    }
}

#[allow(clippy::large_enum_variant)]
enum ManagedScene<'a>{
    Unloaded,
    Loaded(Scene<'a>)
}

#[derive(Default)]
pub struct Scene<'a> {
    world: World<'a>
}

impl<'a> Scene<'a>{
    pub fn step(&mut self){
        self.world.step();
    }
}

