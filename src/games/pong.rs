use ratatui::{Frame, layout::Rect};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{set_char, draw_text, fill_rect};

pub struct PongGame {
    id: String,
    paddle1_y: u16, paddle2_y: u16,
    ball_x: f32, ball_y: f32,
    ball_vx: f32, ball_vy: f32,
    score1: i32, score2: i32,
    speed: f32,
    ai: bool,
    game_over: bool, paused: bool,
    w: u16, h: u16,
    ai_timer: f32,
}

impl PongGame {
    pub fn new() -> Self {
        Self {
            id: "pong".into(), paddle1_y: 10, paddle2_y: 10,
            ball_x: 40.0, ball_y: 12.0, ball_vx: 1.2, ball_vy: 0.6,
            score1: 0, score2: 0, speed: 1.0, ai: true,
            game_over: false, paused: false, w: 80, h: 24,
            ai_timer: 0.0,
        }
    }

    fn reset_ball(&mut self) {
        let angle = (rand::random::<f32>() - 0.5) * std::f32::consts::PI / 4.0;
        self.ball_x = self.w as f32 / 2.0;
        self.ball_y = self.h as f32 / 2.0;
        let dir = if rand::random() { 1.0 } else { -1.0 };
        self.ball_vx = dir * (0.8 + rand::random::<f32>() * 0.4);
        self.ball_vy = angle.sin() * 0.6;
        self.speed = 1.0;
    }
}

impl Scene for PongGame {
    fn id(&self) -> &str { &self.id }
    fn scene_type(&self) -> SceneType { SceneType::Game }
    fn init(&mut self) {
        let paddle_center = self.h / 2 - 2;
        self.paddle1_y = paddle_center;
        self.paddle2_y = paddle_center;
        self.score1 = 0; self.score2 = 0; self.speed = 1.0;
        self.game_over = false; self.paused = false;
        self.ai_timer = 0.0;
        self.reset_ball();
    }
    fn enter(&mut self) { self.paused = false; }

    fn set_terminal_size(&mut self, w: u16, h: u16) {
        self.w = w.max(20);
        self.h = h.max(10);
    }

    fn update(&mut self, dt: f32) {
        if self.paused || self.game_over { return; }
        let dt = dt * 60.0 * self.speed;
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;
        if self.ball_y <= 0.0 || self.ball_y >= self.h as f32 - 1.0 {
            self.ball_vy = -self.ball_vy;
            self.ball_y = self.ball_y.max(1.0).min(self.h as f32 - 2.0);
        }

        let hw = self.w as f32;

        if self.ball_vx < 0.0 && self.ball_x <= 3.0
            && self.ball_y >= self.paddle1_y as f32 - 0.5
            && self.ball_y <= (self.paddle1_y + 5) as f32 + 0.5
        {
            self.ball_vx = self.ball_vx.abs();
            self.ball_vy += (self.ball_y - (self.paddle1_y as f32 + 2.5)) * 0.08;
            let speed = self.ball_vy.abs().min(2.5);
            self.ball_vy = self.ball_vy.signum() * speed.max(0.3);
            self.ball_vx = (self.ball_vx + 0.05).min(2.5);
            self.speed = (self.speed + 0.03).min(2.5);
            self.ball_x = 4.0;
        }

        if self.ball_vx > 0.0 && self.ball_x >= hw - 4.0
            && self.ball_y >= self.paddle2_y as f32 - 0.5
            && self.ball_y <= (self.paddle2_y + 5) as f32 + 0.5
        {
            self.ball_vx = -self.ball_vx.abs();
            self.ball_vy += (self.ball_y - (self.paddle2_y as f32 + 2.5)) * 0.08;
            let speed = self.ball_vy.abs().min(2.5);
            self.ball_vy = self.ball_vy.signum() * speed.max(0.3);
            self.ball_vx = (self.ball_vx.abs() + 0.05).min(2.5) * -1.0;
            self.speed = (self.speed + 0.03).min(2.5);
            self.ball_x = hw - 5.0;
        }

        if self.ball_x < 0.0 { self.score2 += 1; if self.score2 >= 5 { self.game_over = true; return; } self.reset_ball(); }
        if self.ball_x > hw { self.score1 += 1; if self.score1 >= 5 { self.game_over = true; return; } self.reset_ball(); }

        if self.ai {
            self.ai_timer += dt * self.speed;
            if self.ball_vx > 0.0 && self.ai_timer >= 0.08 {
                self.ai_timer = 0.0;
                let target_y = self.ball_y + (rand::random::<f32>() - 0.5) * 4.0;
                let diff = target_y - (self.paddle2_y as f32 + 2.5);
                if diff.abs() > 2.0 {
                    let step = diff.signum() * diff.abs().min(1.5);
                    self.paddle2_y = (self.paddle2_y as f32 + step) as u16;
                }
                self.paddle2_y = self.paddle2_y.clamp(0, self.h.saturating_sub(5));
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let buf = frame.buffer_mut();
        let c = engine.theme.colors();
        fill_rect(buf, area, c.bg);

        let game_w = self.w.min(area.width);
        let game_h = self.h.min(area.height);
        let offset_x = area.x + (area.width.saturating_sub(game_w)) / 2;
        let offset_y = area.y + (area.height.saturating_sub(game_h)) / 2;

        for y in (0..game_h).step_by(2) {
            let sy = offset_y + y;
            if sy < area.y + area.height {
                set_char(buf, offset_x + game_w / 2, sy, '│', c.border, c.bg);
            }
        }
        for i in 0..5 {
            let px = offset_x + 1;
            let py = offset_y + self.paddle1_y + i;
            if px < area.x + area.width && py < area.y + area.height {
                set_char(buf, px, py, '█', c.accent, c.bg);
            }
            let px = offset_x + game_w - 2;
            let py = offset_y + self.paddle2_y + i;
            if px < area.x + area.width && py < area.y + area.height {
                set_char(buf, px, py, '█', c.accent, c.bg);
            }
        }
        let bx = offset_x + (self.ball_x as u16).min(game_w - 1);
        let by = offset_y + (self.ball_y as u16).min(game_h - 1);
        if bx < area.x + area.width && by < area.y + area.height {
            set_char(buf, bx, by, '●', c.secondary, c.bg);
        }
        let score = format!("{}  -  {}", self.score1, self.score2);
        draw_text(buf, offset_x + game_w / 2 - score.len() as u16 / 2, offset_y, &score, c.fg, c.bg);
        if self.game_over {
            let msg = if self.score1 > self.score2 { "Player 1 Wins!" } else { "Player 2 Wins!" };
            draw_text(buf, offset_x + game_w / 2 - 10, offset_y + game_h / 2, msg, c.error, c.bg);
            draw_text(buf, offset_x + game_w / 2 - 15, offset_y + game_h / 2 + 1, "Press SPACE to restart, ESC to exit", c.disabled, c.bg);
        }
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        match key {
            38 | 87 => self.paddle1_y = self.paddle1_y.saturating_sub(2),
            40 | 83 => self.paddle1_y = (self.paddle1_y + 2).min(self.h.saturating_sub(5)),
            32 => if self.game_over { self.init(); } else { self.paused = !self.paused; }
            _ => {}
        }
    }
}
