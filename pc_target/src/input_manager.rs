use citra_engine::input::InputManager;
use winit::application::ApplicationHandler;

pub struct PCInputManager {
    window: Arc<Window>,
}
impl InputManager for PCInputManager {
    fn scan(&mut self) -> citra_engine::input::InputState {}
    fn should_continue(&self) -> bool {}
}
