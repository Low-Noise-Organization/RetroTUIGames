use ratatui::{Frame, layout::Rect, style::Color};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{draw_text, fill_rect, draw_border};

pub struct Game2048 {
    id: String,
    grid: [[u16; 4]; 4],
    score: i32,
    high_score: i32,
    best_tile: u16,
    game_over: bool,
    won: bool,
    keep_playing: bool,
    undo_grid: [[u16; 4]; 4],
    undo_score: i32,
    has_undo: bool,
}

fn tile_color(val: u16) -> (Color, Color) {
    match val {
        0 => (Color::Rgb(60, 58, 50), Color::Rgb(60, 58, 50)),
        2 => (Color::Rgb(238, 228, 218), Color::Rgb(238, 228, 218)),
        4 => (Color::Rgb(237, 224, 200), Color::Rgb(237, 224, 200)),
        8 => (Color::Rgb(242, 177, 121), Color::Rgb(242, 177, 121)),
        16 => (Color::Rgb(245, 149, 99), Color::Rgb(245, 149, 99)),
        32 => (Color::Rgb(246, 124, 95), Color::Rgb(246, 124, 95)),
        64 => (Color::Rgb(246, 94, 59), Color::Rgb(246, 94, 59)),
        128 => (Color::Rgb(237, 207, 114), Color::Rgb(237, 207, 114)),
        256 => (Color::Rgb(237, 204, 97), Color::Rgb(237, 204, 97)),
        512 => (Color::Rgb(237, 200, 80), Color::Rgb(237, 200, 80)),
        1024 => (Color::Rgb(237, 197, 63), Color::Rgb(237, 197, 63)),
        2048 => (Color::Rgb(237, 194, 46), Color::Rgb(237, 194, 46)),
        _ => (Color::Rgb(60, 58, 50), Color::Rgb(237, 194, 46)),
    }
}

impl Game2048 {
    pub fn new() -> Self {
        let mut g = Self {
            id: "2048".into(), grid: [[0; 4]; 4], score: 0, high_score: 0,
            best_tile: 0, game_over: false, won: false, keep_playing: false,
            undo_grid: [[0; 4]; 4], undo_score: 0, has_undo: false,
        };
        g.spawn_tile();
        g.spawn_tile();
        g
    }

    fn spawn_tile(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut empty = Vec::new();
        for y in 0..4 { for x in 0..4 { if self.grid[y][x] == 0 { empty.push((x, y)); } } }
        if empty.is_empty() { return; }
        let (x, y) = empty[rng.gen_range(0..empty.len())];
        self.grid[y][x] = if rng.gen::<f32>() < 0.9 { 2 } else { 4 };
    }

    fn slide_left(&mut self) -> bool {
        let mut moved = false;
        for y in 0..4 {
            let row = self.grid[y];
            let mut new = [0u16; 4];
            let mut pos = 0;
            let mut i = 0;
            while i < 4 {
                if row[i] != 0 {
                    if pos > 0 && new[pos - 1] == row[i] {
                        new[pos - 1] *= 2;
                        self.score += new[pos - 1] as i32;
                        self.best_tile = self.best_tile.max(new[pos - 1]);
                        if new[pos - 1] == 2048 && !self.won { self.won = true; }
                        i += 1;
                    } else {
                        new[pos] = row[i];
                        pos += 1;
                        i += 1;
                    }
                } else { i += 1; }
            }
            if new != row { moved = true; }
            self.grid[y] = new;
        }
        moved
    }

    fn slide_right(&mut self) -> bool {
        self.rotate_cw(); self.rotate_cw();
        let moved = self.slide_left();
        self.rotate_cw(); self.rotate_cw();
        moved
    }

    fn slide_up(&mut self) -> bool {
        self.rotate_cw(); self.rotate_cw(); self.rotate_cw();
        let moved = self.slide_left();
        self.rotate_cw();
        moved
    }

    fn slide_down(&mut self) -> bool {
        self.rotate_cw();
        let moved = self.slide_left();
        self.rotate_cw(); self.rotate_cw(); self.rotate_cw();
        moved
    }

    fn rotate_cw(&mut self) {
        let mut new = [[0u16; 4]; 4];
        for y in 0..4 { for x in 0..4 { new[x][3 - y] = self.grid[y][x]; } }
        self.grid = new;
    }

    fn can_move(&self) -> bool {
        for y in 0..4 { for x in 0..4 {
            if self.grid[y][x] == 0 { return true; }
            if x < 3 && self.grid[y][x] == self.grid[y][x + 1] { return true; }
            if y < 3 && self.grid[y][x] == self.grid[y + 1][x] { return true; }
        }}
        false
    }

    fn do_move(&mut self, dir: i32) {
        self.undo_grid = self.grid;
        self.undo_score = self.score;
        self.has_undo = true;

        let moved = match dir {
            37 => self.slide_left(),
            39 => self.slide_right(),
            38 => self.slide_up(),
            40 => self.slide_down(),
            _ => false,
        };
        if moved { self.spawn_tile(); }
        if !self.keep_playing && !self.can_move() { self.game_over = true; }
    }
}

impl Scene for Game2048 {
    fn id(&self) -> &str { &self.id }
    fn scene_type(&self) -> SceneType { SceneType::Game }
    fn set_terminal_size(&mut self, _w: u16, _h: u16) {}

    fn init(&mut self) {
        self.grid = [[0; 4]; 4]; self.score = 0; self.game_over = false;
        self.won = false; self.keep_playing = false; self.has_undo = false;
        self.best_tile = 0; self.spawn_tile(); self.spawn_tile();
    }
    fn enter(&mut self) {}

    fn update(&mut self, _dt: f32) {}

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let c = engine.theme.colors();
        let buf = frame.buffer_mut();
        fill_rect(buf, area, c.bg);

        let score_text = format!("Score: {}  Best: {}", self.score, self.high_score);
        draw_text(buf, area.x + 1, area.y, &score_text, c.fg, c.bg);

        let total_w = 4 * 7 + 5;
        let start_x = area.x + (area.width.saturating_sub(total_w)) / 2;
        let start_y = area.y + 3;

        let grid_rect = Rect::new(start_x, start_y, total_w, 4 * 3 + 5);
        fill_rect(buf, grid_rect, Color::Rgb(60, 58, 50));
        draw_border(buf, grid_rect, c.border, Color::Rgb(60, 58, 50));

        for y in 0..4 {
            for x in 0..4 {
                let val = self.grid[y][x];
                let (_, bg) = tile_color(val);
                let fg = if val > 4 { Color::Rgb(255, 255, 255) } else { Color::Rgb(60, 58, 50) };
                let cx = start_x + 1 + (x as u16) * 7 + (x as u16);
                let cy = start_y + 1 + (y as u16) * 3 + (y as u16);

                let cell_rect = Rect::new(cx, cy, 6, 2);
                fill_rect(buf, cell_rect, bg);
                if val > 0 {
                    let s = val.to_string();
                    draw_text(buf, cx + (6 - s.len() as u16) / 2, cy + 1, &s, fg, bg);
                }
            }
        }

        if self.won && !self.keep_playing {
            draw_text(buf, area.x + area.width / 2 - 4, area.y + area.height / 2, "YOU WIN!", c.success, c.bg);
            draw_text(buf, area.x + area.width / 2 - 10, area.y + area.height / 2 + 1, "Press SPACE to continue", c.disabled, c.bg);
        }
        if self.game_over {
            draw_text(buf, area.x + area.width / 2 - 5, area.y + area.height / 2, "GAME OVER", c.error, c.bg);
            draw_text(buf, area.x + area.width / 2 - 10, area.y + area.height / 2 + 1, "Press R to restart", c.disabled, c.bg);
        }
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        match key {
            37 | 38 | 39 | 40 => self.do_move(key),
            114 => { self.init(); }
            117 if self.has_undo => {
                self.grid = self.undo_grid;
                self.score = self.undo_score;
                self.has_undo = false;
            }
            32 => if self.won { self.keep_playing = true; }
            27 => {}
            _ => {}
        }
    }
}
