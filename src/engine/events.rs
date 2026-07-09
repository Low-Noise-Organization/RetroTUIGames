use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Key(i32, char, bool, bool, bool),
    Resize(u16, u16),
    Scene(SceneAction, String),
    Game(GameAction, String, Option<i32>),
    Achievement(String, String),
    Tick(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneAction { Enter, Exit, Pause, Resume }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameAction { Start, Stop, Pause, Resume, Score, GameOver, LevelUp }

#[derive(Clone)]
pub struct EventBus {
    tx: mpsc::Sender<Event>,
    rx: std::sync::Arc<std::sync::Mutex<mpsc::Receiver<Event>>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx: std::sync::Arc::new(std::sync::Mutex::new(rx)) }
    }

    pub fn publish(&self, event: Event) {
        let _ = self.tx.send(event);
    }

    pub fn drain(&self) -> Vec<Event> {
        let mut events = Vec::new();
        if let Ok(rx) = self.rx.lock() {
            while let Ok(e) = rx.try_recv() {
                events.push(e);
            }
        }
        events
    }
}
