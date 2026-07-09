use ratatui::{Frame, layout::Rect};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{set_char, draw_text, fill_rect};

pub struct SnakeGame {
    id: String,
    snake: Vec<(u16, u16)>,
    dir: (i32, i32), next_dir: (i32, i32),
    food: (u16, u16),
    w: u16, h: u16,
    score: i32, level: i32,
    move_timer: f32, move_interval: f32,
    game_over: bool, paused: bool, growing: bool,
}

impl SnakeGame {
    pub fn new() -> Self {
        Self {
            id: "snake".into(), snake: Vec::new(), dir: (1, 0), next_dir: (1, 0),
            food: (0, 0), w: 80, h: 24, score: 0, level: 1,
            move_timer: 0.0, move_interval: 0.2,
            game_over: false, paused: false, growing: false,
        }
    }

    fn reset_state(&mut self) {
        let cx = self.w / 2;
        let cy = self.h / 2;
        self.snake = vec![(cx, cy), (cx - 1, cy), (cx - 2, cy)];
        self.dir = (1, 0); self.next_dir = (1, 0);
        self.score = 0; self.level = 1; self.move_timer = 0.0; self.move_interval = 0.2;
        self.game_over = false; self.paused = false; self.growing = false;
        self.spawn_food();
    }

    fn spawn_food(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let fw = self.w.saturating_sub(2);
        let fh = self.h.saturating_sub(2);
        if fw < 1 || fh < 1 { self.game_over = true; return; }
        let max_attempts = 1000;
        for _ in 0..max_attempts {
            let fx = rng.gen_range(1..=fw);
            let fy = rng.gen_range(1..=fh);
            if !self.snake.contains(&(fx, fy)) { self.food = (fx, fy); return; }
        }
        for fy in 1..=fh {
            for fx in 1..=fw {
                if !self.snake.contains(&(fx, fy)) { self.food = (fx, fy); return; }
            }
        }
        self.game_over = true;
    }
}

impl Scene for SnakeGame {
    fn id(&self) -> &str { &self.id }
    fn scene_type(&self) -> SceneType { SceneType::Game }
    fn init(&mut self) { self.reset_state(); }
    fn enter(&mut self) { self.paused = false; }

    fn set_terminal_size(&mut self, w: u16, h: u16) {
        self.w = w.max(16);
        self.h = h.max(10);
    }

    fn update(&mut self, dt: f32) {
        if self.paused || self.game_over { return; }
        self.move_timer += dt;
        if self.move_timer < self.move_interval { return; }
        self.move_timer = 0.0;
        self.dir = self.next_dir;
        let (hx, hy) = self.snake[0];
        let nx = (hx as i32 + self.dir.0) as u16;
        let ny = (hy as i32 + self.dir.1) as u16;
        if nx >= self.w || ny >= self.h || self.snake.contains(&(nx, ny)) { self.game_over = true; return; }
        self.snake.insert(0, (nx, ny));
        if nx == self.food.0 && ny == self.food.1 {
            self.score += self.level * 10; self.growing = true;
            self.level += 1;
            self.move_interval = (self.move_interval * 0.94).max(0.06);
            self.spawn_food();
        }
        if !self.growing { self.snake.pop(); }
        self.growing = false;
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let buf = frame.buffer_mut();
        let c = engine.theme.colors();
        fill_rect(buf, area, c.bg);

        let game_w = self.w.min(area.width.saturating_sub(2));
        let game_h = self.h.min(area.height.saturating_sub(3));
        let offset_x = area.x + (area.width.saturating_sub(game_w)) / 2;
        let offset_y = area.y + 2;

        for x in 0..game_w {
            let top = offset_y;
            let bot = offset_y + game_h - 1;
            if top >= area.y && top < area.y + area.height {
                set_char(buf, offset_x + x, top, '█', c.border, c.bg);
            }
            if bot >= area.y && bot < area.y + area.height {
                set_char(buf, offset_x + x, bot, '█', c.border, c.bg);
            }
        }
        for y in 0..game_h {
            let left = offset_x;
            let right = offset_x + game_w - 1;
            let sy = offset_y + y;
            if sy >= area.y && sy < area.y + area.height {
                set_char(buf, left, sy, '█', c.border, c.bg);
                set_char(buf, right, sy, '█', c.border, c.bg);
            }
        }

        let play_w = game_w.saturating_sub(2);
        let play_h = game_h.saturating_sub(2);
        let play_ox = offset_x + 1;
        let play_oy = offset_y + 1;

        for py in 0..play_h {
            for px in 0..play_w {
                let sx = play_ox + px;
                let sy = play_oy + py;
                if sx < area.x + area.width && sy < area.y + area.height {
                    let dot = if (px + py) % 2 == 0 { '·' } else { ' ' };
                    set_char(buf, sx, sy, dot, c.disabled, c.bg);
                }
            }
        }

        for (i, &(x, y)) in self.snake.iter().enumerate() {
            if x < play_w && y < play_h {
                let sx = play_ox + x;
                let sy = play_oy + y;
                if sx < area.x + area.width && sy < area.y + area.height {
                    let ch = if i == 0 { '■' } else { '●' };
                    let fg = if i == 0 { c.accent } else { c.secondary };
                    set_char(buf, sx, sy, ch, fg, c.bg);
                }
            }
        }

        if self.food.0 < play_w && self.food.1 < play_h {
            let fx = play_ox + self.food.0;
            let fy = play_oy + self.food.1;
            set_char(buf, fx, fy, '★', c.error, c.bg);
        }

        let info = format!("Score: {}  Level: {}  Length: {}", self.score, self.level, self.snake.len());
        draw_text(buf, area.x + 1, area.y, &info, c.fg, c.bg);

        if self.game_over {
            let msg = format!("GAME OVER - Score: {}", self.score);
            draw_text(buf, area.x + area.width / 2 - msg.len() as u16 / 2, area.y + area.height / 2, &msg, c.error, c.bg);
            draw_text(buf, area.x + area.width / 2 - 14, area.y + area.height / 2 + 1, "Press SPACE to restart, ESC to exit", c.disabled, c.bg);
        }
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        match key {
            38 | 87 => if self.dir != (0, 1) { self.next_dir = (0, -1); }
            40 | 83 => if self.dir != (0, -1) { self.next_dir = (0, 1); }
            37 | 65 => if self.dir != (1, 0) { self.next_dir = (-1, 0); }
            39 | 68 => if self.dir != (-1, 0) { self.next_dir = (1, 0); }
            32 => if self.game_over { self.init(); } else { self.paused = !self.paused; }
            _ => {}
        }
    }
}
