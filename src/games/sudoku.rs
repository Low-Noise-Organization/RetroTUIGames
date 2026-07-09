use ratatui::{Frame, layout::Rect};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{set_char, draw_text, fill_rect};
use rand::seq::SliceRandom;

pub struct SudokuGame {
    id: String,
    solution: [[u8; 9]; 9],
    player: [[u8; 9]; 9],
    fixed: [[bool; 9]; 9],
    pencil: [[[bool; 10]; 9]; 9],
    pencil_mode: bool,
    cursor_x: usize, cursor_y: usize,
    diff: usize,
    timer: f32, timer_running: bool,
    won: bool,
}

impl SudokuGame {
    pub fn new() -> Self {
        Self {
            id: "sudoku".into(),
            solution: [[0; 9]; 9], player: [[0; 9]; 9],
            fixed: [[false; 9]; 9], pencil: [[[false; 10]; 9]; 9],
            pencil_mode: false, cursor_x: 0, cursor_y: 0,
            diff: 0, timer: 0.0, timer_running: false, won: false,
        }
    }

    fn generate(&mut self) {
        let mut board = [[0u8; 9]; 9];
        Self::solve_internal(&mut board, 0);
        self.solution = board;
        self.player = board;

        let clues = match self.diff { 0 => 36, 1 => 28, _ => 22 };
        let mut removed = 0;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        while removed < 81 - clues {
            let x = rng.gen_range(0..9);
            let y = rng.gen_range(0..9);
            if self.player[y][x] != 0 {
                self.player[y][x] = 0;
                removed += 1;
            }
        }

        self.pencil = [[[false; 10]; 9]; 9];
        for y in 0..9 {
            for x in 0..9 {
                self.fixed[y][x] = self.player[y][x] != 0;
            }
        }
    }

    fn solve_internal(board: &mut [[u8; 9]; 9], pos: usize) -> bool {
        if pos == 81 { return true; }
        let x = pos % 9;
        let y = pos / 9;
        if board[y][x] != 0 { return Self::solve_internal(board, pos + 1); }
        let mut nums: Vec<u8> = (1..=9).collect();
        nums.shuffle(&mut rand::thread_rng());
        for &n in &nums {
            if Self::is_valid(board, x, y, n) {
                board[y][x] = n;
                if Self::solve_internal(board, pos + 1) { return true; }
                board[y][x] = 0;
            }
        }
        false
    }

    fn is_valid(board: &[[u8; 9]; 9], x: usize, y: usize, n: u8) -> bool {
        for i in 0..9 {
            if board[y][i] == n || board[i][x] == n { return false; }
        }
        let bx = x / 3 * 3; let by = y / 3 * 3;
        for dy in 0..3 { for dx in 0..3 { if board[by + dy][bx + dx] == n { return false; } } }
        true
    }

    fn check_win(&mut self) {
        for y in 0..9 { for x in 0..9 { if self.player[y][x] != self.solution[y][x] { return; } } }
        self.won = true;
        self.timer_running = false;
    }
}

impl Scene for SudokuGame {
    fn id(&self) -> &str { &self.id }
    fn scene_type(&self) -> SceneType { SceneType::Game }
    fn set_terminal_size(&mut self, _w: u16, _h: u16) {}

    fn init(&mut self) {
        self.generate();
        self.timer = 0.0;
        self.timer_running = true;
        self.won = false;
        self.pencil_mode = false;
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn enter(&mut self) {}

    fn update(&mut self, dt: f32) {
        if self.timer_running { self.timer += dt; }
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let c = engine.theme.colors();
        let buf = frame.buffer_mut();
        fill_rect(buf, area, c.bg);

        let timer = format!("{:02}:{:02}", (self.timer as u16) / 60, (self.timer as u16) % 60);
        draw_text(buf, area.right().saturating_sub(8), area.y, &timer, c.fg, c.bg);
        draw_text(buf, area.x + 1, area.y, if self.pencil_mode { "PENCIL" } else { "PLACE" }, c.accent, c.bg);

        let grid_w = 9 * 2 + 4;
        let start_x = area.x + (area.width.saturating_sub(grid_w)) / 2;
        let start_y = area.y + 2;

        for y in 0..13 {
            for x in 0..13 {
                let sx = start_x + x as u16;
                let sy = start_y + y as u16;
                if sx >= area.right() || sy >= area.bottom() { continue; }
                let is_h_div = y % 4 == 0;
                let is_v_div = x % 4 == 0;
                if is_h_div && is_v_div {
                    let ch = match ((x / 4) as u8, (y / 4) as u8) {
                        (0, 0) => '┌', (3, 0) => '┐',
                        (0, 3) => '└', (3, 3) => '┘',
                        (_, 0) => '┬', (_, 3) => '┴',
                        (0, _) => '├', (3, _) => '┤',
                        _ => '┼',
                    };
                    set_char(buf, sx, sy, ch, c.border, c.bg);
                } else if is_h_div {
                    set_char(buf, sx, sy, '─', c.border, c.bg);
                } else if is_v_div {
                    set_char(buf, sx, sy, '│', c.border, c.bg);
                }
            }
        }

        for y in 0..9 {
            for x in 0..9 {
                let cell_x = x / 3;
                let cell_y = y / 3;
                let sx = start_x + 1 + x as u16 + cell_x as u16;
                let sy = start_y + 1 + y as u16 + cell_y as u16;
                if sx >= area.right() || sy >= area.bottom() { continue; }

                let val = self.player[y][x];
                let is_fixed = self.fixed[y][x];
                let is_cursor = x == self.cursor_x && y == self.cursor_y;
                let is_wrong = !is_fixed && val != 0 && val != self.solution[y][x];

                let fg = if is_wrong { c.error } else if is_fixed { c.accent } else { c.fg };
                let bg = if is_cursor { c.selection } else { c.bg };

                if val != 0 {
                    set_char(buf, sx, sy, char::from_digit(val as u32, 10).unwrap(), fg, bg);
                } else if self.pencil_mode {
                    let mut marks = String::new();
                    for n in 1..=9 {
                        if self.pencil[y][x][n] { marks.push(char::from_digit(n as u32, 10).unwrap()); }
                    }
                    if marks.len() <= 4 { draw_text(buf, sx, sy, &marks, c.disabled, bg); }
                }
            }
        }

        if self.won {
            let msg = "YOU WIN!";
            draw_text(buf, area.x + area.width / 2 - 4, area.y + area.height / 2, msg, c.success, c.bg);
        }

        let controls = "Arrows: Move | 1-9: Place | Bksp: Clear | P: Pencil | H: Hint | N: New";
        draw_text(buf, area.x + area.width / 2 - controls.len() as u16 / 2, area.y + area.height.saturating_sub(1), controls, c.disabled, c.bg);
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        match key {
            38 => self.cursor_y = self.cursor_y.saturating_sub(1),
            40 => self.cursor_y = (self.cursor_y + 1).min(8),
            37 => self.cursor_x = self.cursor_x.saturating_sub(1),
            39 => self.cursor_x = (self.cursor_x + 1).min(8),
            _ if key >= 49 && key <= 57 => {
                let n = (key - 48) as u8;
                if self.pencil_mode && !self.fixed[self.cursor_y][self.cursor_x] {
                    self.pencil[self.cursor_y][self.cursor_x][n as usize] ^= true;
                } else if !self.fixed[self.cursor_y][self.cursor_x] {
                    self.player[self.cursor_y][self.cursor_x] = n;
                    self.pencil[self.cursor_y][self.cursor_x] = [false; 10];
                    self.check_win();
                }
            }
            8 | 48 | 127 => {
                if !self.fixed[self.cursor_y][self.cursor_x] { self.player[self.cursor_y][self.cursor_x] = 0; }
            }
            112 => self.pencil_mode = !self.pencil_mode,
            104 => {
                for y in 0..9 { for x in 0..9 {
                    if self.player[y][x] != self.solution[y][x] {
                        self.player[y][x] = self.solution[y][x];
                        self.fixed[y][x] = true;
                        return;
                    }
                }}
            }
            110 => {
                self.diff = (self.diff + 1) % 3;
                self.init();
            }
            27 => {}
            _ => {}
        }
    }
}
