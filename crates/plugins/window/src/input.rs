use std::collections::HashSet;

use ecs::{event::EventReader, ressource::{ResMut, Ressource}};
use glfw::{Key, MouseButton};

use crate::window::{EcsWindowEvent};


#[derive(Default)]
pub struct InputManager {
    pub pressed_keys: HashSet<Key>,
    pub pressed_mouse_button: HashSet<MouseButton>,

    pub mouse_pos: (f64, f64),
    pub mouse_delta: (f64, f64),
}
impl Ressource for InputManager {}

pub fn reset_mouse_delta(mut input: ResMut<InputManager>){
    input.mouse_delta = (0.0, 0.0);
}

pub fn update_input_manager(mut input: ResMut<InputManager>, mut events: EventReader<EcsWindowEvent>){
    for event in events.iter(){
        match event.0 {
            glfw::WindowEvent::Key(key, _scancode, action, _modifier)  => {
                match action {
                    glfw::Action::Press => input.pressed_keys.insert(key),
                    glfw::Action::Release => input.pressed_keys.remove(&key),
                    _ => {false}
                };
            },
            glfw::WindowEvent::CursorPos(x, y) => {
                let mut delta = (input.mouse_pos.0 - x, input.mouse_pos.1 - y);
                // if it's too big, we cut it (maybe a freeze or last mouse pos is incorrect, maybe
                // alt + tab)
                if delta.0 > 100.0 || delta.1 > 100.0 || delta.0 < -100.0 || delta.1 < -100.0{
                    delta = (0.0, 0.0)
                }
                input.mouse_pos = (x, y);
                input.mouse_delta = delta;
            },
            glfw::WindowEvent::MouseButton(button, action, _modifier) => {
                match action{
                    glfw::Action::Press => input.pressed_mouse_button.insert(button),
                    glfw::Action::Release => input.pressed_mouse_button.remove(&button),
                    _ => {false}
                };
            },
            _ => {}
        };
    }
}

