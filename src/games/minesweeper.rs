use ratatui::{Frame, layout::Rect};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{set_char, draw_text, fill_rect};

pub struct MinesweeperGame {
    id: String,
    width: u16, height: u16,
    grid: Vec<Vec<u8>>,
    revealed: Vec<Vec<bool>>,
    flagged: Vec<Vec<bool>>,
    cursor_x: u16, cursor_y: u16,
    mines: i32,
    flag_mode: bool,
    first_click: bool,
    game_over: bool, won: bool,
    timer: f32, timer_running: bool,
    game_started: bool,
    diff: usize,
}

struct DiffConfig {
    target_w: u16,
    target_h: u16,
    mines: i32,
    label: &'static str,
}

const DIFFS: &[DiffConfig] = &[
    DiffConfig { target_w: 6,  target_h: 6,  mines: 4,  label: "Baby" },
    DiffConfig { target_w: 8,  target_h: 8,  mines: 8,  label: "Easy" },
    DiffConfig { target_w: 10, target_h: 10, mines: 15, label: "Medium" },
    DiffConfig { target_w: 16, target_h: 16, mines: 40, label: "Hard" },
    DiffConfig { target_w: 24, target_h: 20, mines: 99, label: "Expert" },
];

impl MinesweeperGame {
    pub fn new() -> Self {
        let cfg = &DIFFS[0];
        Self {
            id: "minesweeper".into(),
            width: cfg.target_w, height: cfg.target_h,
            grid: vec![vec![0; cfg.target_w as usize]; cfg.target_h as usize],
            revealed: vec![vec![false; cfg.target_w as usize]; cfg.target_h as usize],
            flagged: vec![vec![false; cfg.target_w as usize]; cfg.target_h as usize],
            cursor_x: 0, cursor_y: 0,
            mines: cfg.mines, flag_mode: false, first_click: true,
            game_over: false, won: false,
            timer: 0.0, timer_running: false, game_started: false,
            diff: 0,
        }
    }

    fn calc_size(&self, area_w: u16, area_h: u16) -> (u16, u16, i32) {
        let cfg = &DIFFS[self.diff];
        let max_w = (area_w.saturating_sub(2)) / 2;
        let max_h = area_h.saturating_sub(4);
        let w = max_w.min(cfg.target_w);
        let h = max_h.min(cfg.target_h);
        let mines = cfg.mines.min((w * h / 2) as i32);
        (w.max(4), h.max(4), mines.max(1))
    }

    fn place_mines(&mut self, safe_x: u16, safe_y: u16) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let (w, h, m) = (self.width, self.height, self.mines);
        let mut placed = 0;
        while placed < m {
            let x = rng.gen_range(0..w);
            let y = rng.gen_range(0..h);
            if (x == safe_x || x == safe_x.wrapping_sub(1) || x == safe_x + 1) &&
               (y == safe_y || y == safe_y.wrapping_sub(1) || y == safe_y + 1) { continue; }
            if self.grid[y as usize][x as usize] != 9 {
                self.grid[y as usize][x as usize] = 9;
                placed += 1;
            }
        }
        for y in 0..h {
            for x in 0..w {
                if self.grid[y as usize][x as usize] == 9 { continue; }
                let mut count = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 &&
                           self.grid[ny as usize][nx as usize] == 9 { count += 1; }
                    }
                }
                self.grid[y as usize][x as usize] = count;
            }
        }
    }

    fn reveal(&mut self, x: u16, y: u16) {
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            if cx >= self.width || cy >= self.height || self.revealed[cy as usize][cx as usize] || self.flagged[cy as usize][cx as usize] {
                continue;
            }
            self.revealed[cy as usize][cx as usize] = true;
            if self.grid[cy as usize][cx as usize] == 9 {
                self.game_over = true;
                return;
            }
            if self.grid[cy as usize][cx as usize] == 0 {
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        stack.push(((cx as i32 + dx) as u16, (cy as i32 + dy) as u16));
                    }
                }
            }
        }
    }

    fn chord(&mut self, x: u16, y: u16) {
        if !self.revealed[y as usize][x as usize] { return; }
        let cell = self.grid[y as usize][x as usize];
        if cell == 0 || cell == 9 { return; }
        let mut flag_count = 0;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = (x as i32 + dx) as u16;
                let ny = (y as i32 + dy) as u16;
                if nx < self.width && ny < self.height && self.flagged[ny as usize][nx as usize] {
                    flag_count += 1;
                }
            }
        }
        if flag_count == cell as i32 {
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = (x as i32 + dx) as u16;
                    let ny = (y as i32 + dy) as u16;
                    self.reveal(nx, ny);
                }
            }
        }
    }

    fn apply_diff(&mut self) {
        let (w, h, m) = self.calc_size(80, 24);
        self.width = w;
        self.height = h;
        self.mines = m;
    }

    fn check_win(&mut self) {
        let mut all_revealed = true;
        let mut all_mines_flagged = true;
        let mut wrong_flags = false;

        for y in 0..self.height {
            for x in 0..self.width {
                let is_mine = self.grid[y as usize][x as usize] == 9;
                let is_revealed = self.revealed[y as usize][x as usize];
                let is_flagged = self.flagged[y as usize][x as usize];

                if !is_mine && !is_revealed {
                    all_revealed = false;
                }
                if is_mine && !is_flagged {
                    all_mines_flagged = false;
                }
                if !is_mine && is_flagged {
                    wrong_flags = true;
                }
            }
        }

        if (all_revealed || (all_mines_flagged && !wrong_flags)) && !self.game_over {
            self.won = true;
            self.game_over = true;
            self.timer_running = false;
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.grid[y as usize][x as usize] == 9 {
                        self.flagged[y as usize][x as usize] = true;
                    }
                }
            }
        }
    }
}

impl Scene for MinesweeperGame {
    fn id(&self) -> &str { &self.id }
    fn scene_type(&self) -> SceneType { SceneType::Game }

    fn set_terminal_size(&mut self, w: u16, h: u16) {
        let (nw, nh, nm) = self.calc_size(w, h);
        self.width = nw;
        self.height = nh;
        self.mines = nm;
    }

    fn init(&mut self) {
        self.grid = vec![vec![0; self.width as usize]; self.height as usize];
        self.revealed = vec![vec![false; self.width as usize]; self.height as usize];
        self.flagged = vec![vec![false; self.width as usize]; self.height as usize];
        self.cursor_x = 0; self.cursor_y = 0;
        self.first_click = true; self.game_over = false; self.won = false;
        self.timer = 0.0; self.timer_running = false; self.game_started = false;
    }

    fn enter(&mut self) {}

    fn update(&mut self, dt: f32) {
        if self.timer_running { self.timer += dt; }
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let c = engine.theme.colors();
        let buf = frame.buffer_mut();
        fill_rect(buf, area, c.bg);

        let (w, h) = (self.width, self.height);
        let remaining_mines = self.mines - self.flagged.iter().flatten().filter(|&&f| f).count() as i32;

        let diff_label = DIFFS[self.diff].label;
        let info = format!("Mines: {}  Time: {:.0}s  [{}]", remaining_mines, self.timer, diff_label);
        draw_text(buf, area.x + 1, area.y, &info, c.fg, c.bg);

        let mode_str = if self.flag_mode {
            "⚑ FLAG (f)"
        } else {
            "⛏ DIG (f)"
        };
        draw_text(buf, area.right().saturating_sub(12), area.y, mode_str, if self.flag_mode { c.warning } else { c.accent }, c.bg);

        let start_x = area.x + (area.width.saturating_sub(w * 2)) / 2;
        let start_y = area.y + 2;

        for y in 0..h {
            for x in 0..w {
                let sx = start_x + x * 2;
                let sy = start_y + y;
                if sx >= area.right() || sy >= area.bottom() { continue; }

                let is_mine = self.grid[y as usize][x as usize] == 9;
                let is_revealed = self.revealed[y as usize][x as usize];
                let is_flagged = self.flagged[y as usize][x as usize];

                let ch = if self.won || (is_revealed && !is_mine) {
                    if is_mine { '⚑' }
                    else {
                        match self.grid[y as usize][x as usize] {
                            0 => ' ',
                            1 => '①', 2 => '②', 3 => '③', 4 => '④',
                            5 => '⑤', 6 => '⑥', 7 => '⑦', 8 => '⑧',
                            _ => '?',
                        }
                    }
                } else if is_flagged {
                    if self.game_over && !self.won && is_mine { '✓' } else { '⚑' }
                } else if self.game_over && !self.won && is_mine {
                    '●'
                } else {
                    '░'
                };

                let is_cursor = x == self.cursor_x && y == self.cursor_y;
                let fg = if is_cursor {
                    c.accent
                } else if self.won {
                    c.success
                } else if is_revealed && is_mine {
                    c.error
                } else if is_flagged {
                    c.warning
                } else {
                    c.fg
                };
                let bg = if is_cursor { c.selection } else if self.won { c.bg } else { c.bg };
                set_char(buf, sx, sy, ch, fg, bg);
            }
        }

        if self.won {
            let msg = format!("YOU WIN!  Time: {:.0}s", self.timer);
            draw_text(buf, area.x + area.width / 2 - msg.len() as u16 / 2, area.y + area.height / 2, &msg, c.success, c.bg);
        } else if self.game_over && !self.won {
            let msg = "GAME OVER";
            draw_text(buf, area.x + area.width / 2 - 5, area.y + area.height / 2, msg, c.error, c.bg);
        }

        let controls = "Arrows: Move  |  Enter/Space: Dig/Flag  |  F: Toggle mode  |  1-5: Difficulty";
        draw_text(buf, area.x + area.width / 2 - controls.len() as u16 / 2, area.y + area.height.saturating_sub(1), controls, c.disabled, c.bg);
    }

    fn handle_key(&mut self, key: i32, ch: char) {
        match key {
            38 => self.cursor_y = self.cursor_y.saturating_sub(1),
            40 => self.cursor_y = (self.cursor_y + 1).min(self.height - 1),
            37 => self.cursor_x = self.cursor_x.saturating_sub(1),
            39 => self.cursor_x = (self.cursor_x + 1).min(self.width - 1),
            10 | 32 => {
                if self.game_over { self.init(); return; }
                if self.first_click {
                    self.place_mines(self.cursor_x, self.cursor_y);
                    self.first_click = false;
                    self.timer_running = true;
                }
                if self.flag_mode {
                    let fl = &mut self.flagged[self.cursor_y as usize][self.cursor_x as usize];
                    *fl = !*fl;
                } else {
                    if self.revealed[self.cursor_y as usize][self.cursor_x as usize] {
                        self.chord(self.cursor_x, self.cursor_y);
                    } else {
                        self.reveal(self.cursor_x, self.cursor_y);
                    }
                }
                if !self.game_over { self.check_win(); }
            }
            102 => self.flag_mode = !self.flag_mode,
            49 => { self.diff = 0; self.apply_diff(); self.init(); }
            50 => { self.diff = 1; self.apply_diff(); self.init(); }
            51 => { self.diff = 2; self.apply_diff(); self.init(); }
            52 => { self.diff = 3; self.apply_diff(); self.init(); }
            53 => { self.diff = 4; self.apply_diff(); self.init(); }
            27 => {}
            _ => {}
        }
    }
}
