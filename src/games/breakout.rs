use crate::engine::renderer::{draw_text, fill_rect, set_char};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use ratatui::{layout::Rect, style::Color, Frame};

const PADDLE_NORMAL: u16 = 7;
const PADDLE_WIDE: u16 = 11;
const WIDE_DURATION: f32 = 10.0;
const BRICK_TOP: u16 = 2;
const BRICK_GAP: u16 = 2;
const BRICK_LEFT: u16 = 1;
const POWER_UP_SPEED: f32 = 0.3;
const MAX_BALLS: usize = 10;

#[derive(Clone, Copy)]
struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

#[derive(Clone, Copy)]
enum PowerUpKind {
    Wider,
    MultiBall,
    ExtraLife,
}

struct PowerUp {
    x: f32,
    y: f32,
    kind: PowerUpKind,
}

pub struct BreakoutGame {
    id: String,
    paddle_x: u16,
    paddle_w: u16,
    balls: Vec<Ball>,
    ball_launched: bool,
    bricks: Vec<Vec<u8>>,
    brick_rows: u16,
    brick_cols: u16,
    lives: i32,
    score: i32,
    level: i32,
    w: u16, h: u16,
    game_over: bool,
    paused: bool,
    power_ups: Vec<PowerUp>,
    wide_timer: f32,
    speed: f32,
}

impl BreakoutGame {
    pub fn new() -> Self {
        Self {
            id: "breakout".into(),
            paddle_x: 0,
            paddle_w: PADDLE_NORMAL,
            balls: vec![Ball {
                x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
            }],
            ball_launched: false,
            bricks: Vec::new(),
            brick_rows: 4,
            brick_cols: 0,
            lives: 3,
            score: 0,
            level: 1,
            w: 80, h: 24,
            game_over: false,
            paused: false,
            power_ups: Vec::new(),
            wide_timer: 0.0,
            speed: 1.0,
        }
    }

    fn paddle_y(&self) -> u16 {
        self.h.saturating_sub(2)
    }

    fn brick_y(row: u16) -> u16 {
        BRICK_TOP + row
    }

    fn brick_x(col: u16) -> u16 {
        BRICK_LEFT + col * BRICK_GAP
    }

    fn brick_color(row: u16) -> Color {
        match row {
            0 => Color::Rgb(255, 85, 85),
            1 => Color::Rgb(255, 170, 85),
            2 => Color::Rgb(255, 255, 85),
            3 => Color::Rgb(85, 255, 85),
            4 => Color::Rgb(85, 170, 255),
            _ => Color::Rgb(170, 85, 255),
        }
    }

    fn brick_score(row: u16) -> i32 {
        match row {
            0 => 50,
            1 => 40,
            2 => 30,
            3 => 20,
            4 => 10,
            _ => 5,
        }
    }

    fn rows_for_level(level: i32) -> u16 {
        match level {
            1 => 4,
            2 => 5,
            _ => 6,
        }
    }

    fn speed_for_level(level: i32) -> f32 {
        1.0 + (level - 1) as f32 * 0.2
    }

    fn reset_level(&mut self) {
        self.brick_rows = Self::rows_for_level(self.level);
        self.brick_cols = (self.w.saturating_sub(2)) / BRICK_GAP;
        if self.brick_cols == 0 {
            self.brick_cols = 1;
        }
        self.bricks.clear();
        for _ in 0..self.brick_rows {
            let mut row = Vec::with_capacity(self.brick_cols as usize);
            for _ in 0..self.brick_cols {
                row.push(1);
            }
            self.bricks.push(row);
        }
    }

    fn launch_ball(&mut self) {
        self.ball_launched = true;
        for ball in &mut self.balls {
            let angle = (rand::random::<f32>() - 0.5) * std::f32::consts::PI * 0.6;
            ball.vx = angle.sin() * 1.5;
            ball.vy = -angle.cos().abs() * 1.5;
        }
    }

    fn reset_balls(&mut self) {
        let py = self.paddle_y();
        let px = self.paddle_x as f32 + self.paddle_w as f32 / 2.0;
        self.balls.clear();
        self.balls.push(Ball {
            x: px,
            y: py as f32 - 1.0,
            vx: 0.0,
            vy: 0.0,
        });
        self.ball_launched = false;
    }

    fn spawn_power_up(&mut self, x: f32, y: f32) {
        if rand::random::<f32>() < 0.2 {
            let kinds = [
                PowerUpKind::Wider,
                PowerUpKind::MultiBall,
                PowerUpKind::ExtraLife,
            ];
            let kind = kinds[rand::random::<usize>() % 3];
            self.power_ups.push(PowerUp { x, y, kind });
        }
    }

    fn apply_power_up(&mut self, kind: PowerUpKind) {
        match kind {
            PowerUpKind::Wider => {
                self.paddle_w = PADDLE_WIDE;
                self.wide_timer = WIDE_DURATION;
                if self.paddle_x + self.paddle_w > self.w {
                    self.paddle_x = self.w.saturating_sub(self.paddle_w);
                }
            }
            PowerUpKind::MultiBall => {
                if self.balls.len() >= MAX_BALLS {
                    return;
                }
                if let Some(ball) = self.balls.first().copied() {
                    let spd = (ball.vx * ball.vx + ball.vy * ball.vy).sqrt().max(0.5);
                    for i in 0..2 {
                        let spread = (i as f32 - 0.5) * 0.6;
                        self.balls.push(Ball {
                            x: ball.x,
                            y: ball.y,
                            vx: ball.vx + spread * spd,
                            vy: ball.vy,
                        });
                    }
                }
            }
            PowerUpKind::ExtraLife => {
                self.lives += 1;
            }
        }
    }
}

impl Scene for BreakoutGame {
    fn id(&self) -> &str { &self.id }
    fn scene_type(&self) -> SceneType { SceneType::Game }

    fn set_terminal_size(&mut self, w: u16, h: u16) {
        self.w = w.max(20);
        self.h = h.max(12);
    }

    fn init(&mut self) {
        self.paddle_x = self.w / 2 - PADDLE_NORMAL / 2;
        self.paddle_w = PADDLE_NORMAL;
        self.lives = 3;
        self.score = 0;
        self.level = 1;
        self.speed = Self::speed_for_level(1);
        self.game_over = false;
        self.paused = false;
        self.power_ups.clear();
        self.wide_timer = 0.0;
        self.reset_level();
        self.reset_balls();
    }

    fn enter(&mut self) { self.paused = false; }

    fn update(&mut self, dt: f32) {
        if self.paused || self.game_over { return; }

        let (w, h) = (self.w, self.h);
        let py = self.paddle_y();

        if !self.ball_launched {
            for ball in &mut self.balls {
                ball.x = self.paddle_x as f32 + self.paddle_w as f32 / 2.0;
                ball.y = py as f32 - 1.0;
            }
        }

        if self.wide_timer > 0.0 {
            self.wide_timer -= dt;
            if self.wide_timer <= 0.0 {
                self.paddle_w = PADDLE_NORMAL;
                if self.paddle_x + self.paddle_w > w {
                    self.paddle_x = w.saturating_sub(self.paddle_w);
                }
            }
        }

        for pu in &mut self.power_ups {
            pu.y += POWER_UP_SPEED * dt * 60.0;
        }

        let mut pu_collect: Vec<usize> = Vec::new();
        for (i, pu) in self.power_ups.iter().enumerate() {
            if pu.y >= h as f32 {
                pu_collect.push(i);
            } else {
                let px = pu.x as u16;
                let py_pu = pu.y as u16;
                if py_pu >= py && px >= self.paddle_x && px < self.paddle_x + self.paddle_w {
                    pu_collect.push(i);
                }
            }
        }
        for &i in pu_collect.iter().rev() {
            if i < self.power_ups.len() {
                let kind = self.power_ups[i].kind;
                self.apply_power_up(kind);
                self.power_ups.swap_remove(i);
            }
        }

        let dt_factor = dt * 60.0 * self.speed;
        let mut dead_balls: Vec<usize> = Vec::new();
        let mut power_spawns: Vec<(f32, f32)> = Vec::new();

        for bi in 0..self.balls.len() {
            let ball = &mut self.balls[bi];
            ball.x += ball.vx * dt_factor;
            ball.y += ball.vy * dt_factor;

            if ball.x <= 0.0 {
                ball.x = 0.0;
                ball.vx = ball.vx.abs();
            }
            if ball.x >= w as f32 - 1.0 {
                ball.x = w as f32 - 1.0;
                ball.vx = -ball.vx.abs();
            }
            if ball.y <= 1.0 {
                ball.y = 1.0;
                ball.vy = ball.vy.abs();
            }

            if ball.vy > 0.0 {
                let bx = ball.x as u16;
                let by = ball.y as u16;
                if bx >= self.paddle_x && bx < self.paddle_x + self.paddle_w && by >= py {
                    ball.vy = -(ball.vy.abs().min(self.speed * 3.0));
                    let hit = (ball.x - self.paddle_x as f32) / self.paddle_w as f32 - 0.5;
                    ball.vx = (ball.vx + hit * self.speed * 2.0).clamp(-3.0, 3.0);
                    ball.y = py as f32 - 1.0;
                } else if by >= h {
                    dead_balls.push(bi);
                    continue;
                }
            }

            'bricks: for row in 0..self.brick_rows {
                for col in 0..self.brick_cols {
                    let r = row as usize;
                    let c = col as usize;
                    if r >= self.bricks.len() || c >= self.bricks[r].len() {
                        continue;
                    }
                    if self.bricks[r][c] == 0 {
                        continue;
                    }

                    let bx = Self::brick_x(col);
                    let by = Self::brick_y(row);
                    let bix = ball.x as u16;
                    let biy = ball.y as u16;

                    if bix == bx && biy == by {
                        self.bricks[r][c] = 0;
                        self.score += Self::brick_score(row);
                        power_spawns.push((bx as f32, by as f32));
                        let dx = ball.x - (bx as f32 + 0.5);
                        let dy = ball.y - (by as f32 + 0.5);
                        if dx.abs() > dy.abs() {
                            ball.vx = -ball.vx;
                            ball.x += ball.vx.signum() * 0.5;
                        } else {
                            ball.vy = -ball.vy;
                            ball.y += ball.vy.signum() * 0.5;
                        }
                        break 'bricks;
                    }
                }
            }
        }

        for &(px, py) in &power_spawns {
            self.spawn_power_up(px, py);
        }

        for &i in dead_balls.iter().rev() {
            if i < self.balls.len() {
                self.balls.swap_remove(i);
            }
        }

        if self.balls.is_empty() {
            self.lives -= 1;
            if self.lives <= 0 {
                self.game_over = true;
            } else {
                self.reset_balls();
            }
            return;
        }

        if self.bricks.iter().all(|row| row.iter().all(|&b| b == 0)) {
            self.level += 1;
            self.speed = Self::speed_for_level(self.level);
            self.reset_level();
            self.reset_balls();
        }
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let buf = frame.buffer_mut();
        let (w, h) = (self.w.min(area.width), self.h.min(area.height));
        let c = engine.theme.colors();
        let ox = area.x + (area.width.saturating_sub(w)) / 2;
        let oy = area.y + (area.height.saturating_sub(h)) / 2;

        fill_rect(buf, Rect::new(ox, oy, w, h), c.bg);

        let info = format!("Score: {}  Lives: {}  Level: {}", self.score, self.lives, self.level);
        draw_text(buf, ox + 1, oy, &info, c.fg, c.bg);
        if self.paused {
            draw_text(buf, ox + w / 2 - 3, oy, "PAUSED", c.warning, c.bg);
        }

        for row in 0..self.brick_rows {
            for col in 0..self.brick_cols {
                let r = row as usize;
                let c_b = col as usize;
                if r < self.bricks.len() && c_b < self.bricks[r].len() && self.bricks[r][c_b] > 0 {
                    let bx = ox + Self::brick_x(col);
                    let by = oy + Self::brick_y(row);
                    if bx < ox + w && by < oy + h {
                        set_char(buf, bx, by, '█', Self::brick_color(row), c.bg);
                    }
                }
            }
        }

        let py = oy + self.paddle_y();
        for i in 0..self.paddle_w {
            let px = ox + self.paddle_x + i;
            if px < ox + w && py < oy + h {
                set_char(buf, px, py, '▄', c.accent, c.bg);
            }
        }

        for ball in &self.balls {
            let bx = ox + (ball.x as u16).min(w - 1);
            let by = oy + (ball.y as u16).min(h - 1);
            if bx < ox + w && by < oy + h {
                set_char(buf, bx, by, '●', c.secondary, c.bg);
            }
        }

        for pu in &self.power_ups {
            let px = ox + (pu.x as u16).min(w - 1);
            let py_pu = oy + (pu.y as u16).min(h - 1);
            if px < ox + w && py_pu < oy + h {
                let (ch, col) = match pu.kind {
                    PowerUpKind::Wider => ('W', Color::Rgb(0, 255, 255)),
                    PowerUpKind::MultiBall => ('M', Color::Rgb(255, 0, 255)),
                    PowerUpKind::ExtraLife => ('L', Color::Rgb(0, 255, 0)),
                };
                set_char(buf, px, py_pu, ch, col, c.bg);
            }
        }

        if self.game_over {
            let msg = format!("GAME OVER - Score: {}", self.score);
            draw_text(buf, ox + w / 2 - msg.len() as u16 / 2, oy + h / 2, &msg, c.error, c.bg);
            draw_text(buf, ox + w / 2 - 15, oy + h / 2 + 1, "Press SPACE to restart, ESC to exit", c.disabled, c.bg);
        }
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        match key {
            37 => { self.paddle_x = self.paddle_x.saturating_sub(2); }
            39 => { self.paddle_x = (self.paddle_x + 2).min(self.w.saturating_sub(self.paddle_w)); }
            32 => {
                if self.game_over { self.init(); }
                else if !self.ball_launched { self.launch_ball(); }
                else { self.paused = !self.paused; }
            }
            27 => {}
            _ => {}
        }
    }
}
