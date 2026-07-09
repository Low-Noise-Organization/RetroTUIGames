pub struct Timer {
    elapsed: f32,
    running: bool,
}

impl Timer {
    pub fn new() -> Self { Self { elapsed: 0.0, running: false } }
    pub fn start(&mut self) { self.running = true; }
    pub fn stop(&mut self) { self.running = false; }
    pub fn reset(&mut self) { self.elapsed = 0.0; }
    pub fn update(&mut self, dt: f32) { if self.running { self.elapsed += dt; } }
    pub fn elapsed(&self) -> f32 { self.elapsed }
    pub fn is_running(&self) -> bool { self.running }
}

pub struct GameLoop {
    pub target_fps: f32,
    pub frame_time: f32,
    pub running: bool,
}

impl GameLoop {
    pub fn new(fps: u32) -> Self {
        let ft = 1.0 / fps as f32;
        Self { target_fps: fps as f32, frame_time: ft, running: false }
    }
}
