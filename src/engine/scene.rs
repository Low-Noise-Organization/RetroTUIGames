use ratatui::{Frame, layout::Rect};
use crate::engine::Engine;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneType {
    Splash, MainMenu, Game, Settings, Achievements, Leaderboard, Exit,
}

pub trait Scene {
    fn id(&self) -> &str;
    fn scene_type(&self) -> SceneType;
    fn init(&mut self) {}
    fn enter(&mut self) {}
    fn exit(&mut self) {}
    fn pause(&mut self) {}
    fn resume(&mut self) {}
    fn update(&mut self, dt: f32);
    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect);
    fn handle_key(&mut self, key: i32, ch: char);
    fn set_terminal_size(&mut self, _w: u16, _h: u16) {}
}

pub struct SceneManager {
    stack: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    pub fn new() -> Self { Self { stack: Vec::new() } }

    pub fn push(&mut self, scene: Box<dyn Scene>) {
        if let Some(current) = self.stack.last_mut() { current.pause(); }
        self.stack.push(scene);
    }

    pub fn pop(&mut self) {
        if let Some(mut exiting) = self.stack.pop() { exiting.exit(); }
        if let Some(current) = self.stack.last_mut() { current.resume(); }
    }

    pub fn current(&self) -> Option<&dyn Scene> { self.stack.last().map(|s| s.as_ref()) }
    pub fn current_mut(&mut self) -> Option<&mut (dyn Scene + 'static)> {
        self.stack.last_mut().map(|s| s.as_mut())
    }
    pub fn is_empty(&self) -> bool { self.stack.is_empty() }
    pub fn clear(&mut self) { while let Some(mut s) = self.stack.pop() { s.exit(); } }
}
