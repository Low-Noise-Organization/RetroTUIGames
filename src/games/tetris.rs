use ratatui::{Frame, layout::Rect, style::Color};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{set_char, draw_text, fill_rect, draw_border};
use rand::seq::SliceRandom;

const COLS: usize = 10;
const ROWS: usize = 20;
const BOARD_W: u16 = 12;
const BOARD_H: u16 = 22;
const BLOCK: char = '█';
const GHOST: char = '▒';

const SHAPES: [[[bool; 4]; 4]; 7] = [
    [
        [false, false, false, false],
        [true,  true,  true,  true],
        [false, false, false, false],
        [false, false, false, false],
    ],
    [
        [false, false, false, false],
        [false, true,  true,  false],
        [false, true,  true,  false],
        [false, false, false, false],
    ],
    [
        [false, false, false, false],
        [false, true,  false, false],
        [true,  true,  true,  false],
        [false, false, false, false],
    ],
    [
        [false, false, false, false],
        [false, true,  true,  false],
        [true,  true,  false, false],
        [false, false, false, false],
    ],
    [
        [false, false, false, false],
        [true,  true,  false, false],
        [false, true,  true,  false],
        [false, false, false, false],
    ],
    [
        [false, false, false, false],
        [true,  false, false, false],
        [true,  true,  true,  false],
        [false, false, false, false],
    ],
    [
        [false, false, false, false],
        [false, false, true,  false],
        [true,  true,  true,  false],
        [false, false, false, false],
    ],
];

pub fn piece_color(piece: u8) -> Color {
    match piece {
        1 => Color::Cyan,
        2 => Color::Yellow,
        3 => Color::Magenta,
        4 => Color::Green,
        5 => Color::Red,
        6 => Color::Blue,
        7 => Color::Rgb(255, 165, 0),
        _ => Color::White,
    }
}

fn get_rotated(base: &[[bool; 4]; 4], rotation: u8) -> Vec<Vec<bool>> {
    let mut cells = vec![vec![false; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            cells[r][c] = base[r][c];
        }
    }
    for _ in 0..rotation {
        let mut next = vec![vec![false; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                next[r][c] = cells[3 - c][r];
            }
        }
        cells = next;
    }
    cells
}

fn rotated_cells(cells: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut next = vec![vec![false; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            next[r][c] = cells[3 - c][r];
        }
    }
    next
}

fn collides(grid: &[Vec<u8>], cells: &[Vec<bool>], x: i32, y: i32) -> bool {
    for r in 0..4 {
        for c in 0..4 {
            if cells[r][c] {
                let bx = x + c as i32;
                let by = y + r as i32;
                if bx < 0 || bx >= COLS as i32 || by >= ROWS as i32 {
                    return true;
                }
                if by >= 0 && grid[by as usize][bx as usize] != 0 {
                    return true;
                }
            }
        }
    }
    false
}

fn lock_piece(grid: &mut [Vec<u8>], piece: &Piece) {
    for r in 0..4 {
        for c in 0..4 {
            if piece.cells[r][c] {
                let bx = piece.x + c as i32;
                let by = piece.y + r as i32;
                if bx >= 0 && bx < COLS as i32 && by >= 0 && by < ROWS as i32 {
                    grid[by as usize][bx as usize] = piece.typ;
                }
            }
        }
    }
}

fn clear_lines(grid: &mut Vec<Vec<u8>>) -> i32 {
    let mut cleared = 0;
    let mut y = 0;
    while y < ROWS {
        if grid[y].iter().all(|&c| c != 0) {
            grid.remove(y);
            grid.insert(0, vec![0u8; COLS]);
            cleared += 1;
        } else {
            y += 1;
        }
    }
    cleared
}

fn ghost_y(grid: &[Vec<u8>], cells: &[Vec<bool>], x: i32, y: i32) -> i32 {
    let mut gy = y;
    while !collides(grid, cells, x, gy + 1) {
        gy += 1;
    }
    gy
}

fn line_score(lines: i32, level: i32) -> i32 {
    (match lines {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    }) * level
}

fn drop_interval_for_level(level: i32) -> f32 {
    (0.8 - (level as f32 - 1.0) * 0.07).max(0.05)
}

pub struct Piece {
    typ: u8,
    rotation: u8,
    x: i32,
    y: i32,
    cells: Vec<Vec<bool>>,
}

pub struct TetrisGame {
    id: String,
    grid: Vec<Vec<u8>>,
    current: Piece,
    next_queue: Vec<Piece>,
    held: Option<u8>,
    hold_used: bool,
    score: i32,
    level: i32,
    lines: i32,
    game_over: bool,
    paused: bool,
    drop_timer: f32,
    drop_interval: f32,
    bag: Vec<u8>,
}

impl TetrisGame {
    pub fn new() -> Self {
        let mut s = Self {
            id: "tetris".into(),
            grid: vec![vec![0u8; COLS]; ROWS],
            current: Piece {
                typ: 1, rotation: 0, x: 3, y: 0,
                cells: get_rotated(&SHAPES[0], 0),
            },
            next_queue: Vec::new(),
            held: None,
            hold_used: false,
            score: 0,
            level: 1,
            lines: 0,
            game_over: false,
            paused: false,
            drop_timer: 0.0,
            drop_interval: 1.0,
            bag: Vec::new(),
        };
        s.fill_bag();
        s.fill_queue();
        s.spawn_next();
        s
    }

    fn fill_bag(&mut self) {
        let mut rng = rand::thread_rng();
        let mut bag: Vec<u8> = (1..=7).collect();
        bag.shuffle(&mut rng);
        self.bag.extend(bag);
    }

    fn fill_queue(&mut self) {
        while self.next_queue.len() < 3 {
            if self.bag.is_empty() {
                self.fill_bag();
            }
            let typ = self.bag.remove(0);
            let cells = get_rotated(&SHAPES[typ as usize - 1], 0);
            self.next_queue.push(Piece { typ, rotation: 0, x: 0, y: 0, cells });
        }
    }

    fn spawn_next(&mut self) {
        self.fill_queue();
        if self.next_queue.is_empty() {
            return;
        }
        let mut piece = self.next_queue.remove(0);
        piece.x = 3;
        piece.y = 0;
        self.current = piece;
        self.fill_queue();
        if collides(&self.grid, &self.current.cells, self.current.x, self.current.y) {
            self.game_over = true;
        }
    }

    fn lock_and_spawn(&mut self) {
        lock_piece(&mut self.grid, &self.current);
        let cleared = clear_lines(&mut self.grid);
        if cleared > 0 {
            self.lines += cleared;
            self.score += line_score(cleared, self.level);
            let new_level = self.lines / 10 + 1;
            if new_level > self.level {
                self.level = new_level;
                self.drop_interval = drop_interval_for_level(self.level);
            }
        }
        self.hold_used = false;
        self.spawn_next();
    }

    fn hold_piece(&mut self) {
        if self.hold_used {
            return;
        }
        let typ = self.current.typ;
        if let Some(held_typ) = self.held {
            let cells = get_rotated(&SHAPES[held_typ as usize - 1], 0);
            self.current = Piece {
                typ: held_typ,
                rotation: 0,
                x: 3,
                y: 0,
                cells,
            };
            self.held = Some(typ);
        } else {
            self.held = Some(typ);
            self.spawn_next();
        }
        self.hold_used = true;
    }
}

impl Scene for TetrisGame {
    fn id(&self) -> &str {
        &self.id
    }

    fn scene_type(&self) -> SceneType {
        SceneType::Game
    }

    fn set_terminal_size(&mut self, _w: u16, _h: u16) {}

    fn init(&mut self) {
        self.grid = vec![vec![0u8; COLS]; ROWS];
        self.score = 0;
        self.level = 1;
        self.lines = 0;
        self.game_over = false;
        self.paused = false;
        self.drop_timer = 0.0;
        self.drop_interval = 1.0;
        self.hold_used = false;
        self.held = None;
        self.bag.clear();
        self.next_queue.clear();
        self.fill_bag();
        self.fill_queue();
        self.spawn_next();
    }

    fn update(&mut self, dt: f32) {
        if self.paused || self.game_over {
            return;
        }
        self.drop_timer += dt;
        if self.drop_timer >= self.drop_interval {
            self.drop_timer = 0.0;
            if !collides(&self.grid, &self.current.cells, self.current.x, self.current.y + 1) {
                self.current.y += 1;
            } else {
                self.lock_and_spawn();
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let buf = frame.buffer_mut();
        let colors = engine.theme.colors();
        fill_rect(buf, area, colors.bg);

        let board_y = (area.height.saturating_sub(BOARD_H)) / 2;
        let board_x = area.x + (area.width.saturating_sub(35).min(area.width.saturating_sub(26))) / 2;

        let board_rect = Rect::new(board_x, board_y, BOARD_W, BOARD_H);
        draw_border(buf, board_rect, colors.border, colors.bg);

        for row in 0..ROWS {
            for col in 0..COLS {
                if self.grid[row][col] != 0 {
                    let px = board_x + 1 + col as u16;
                    let py = board_y + 1 + row as u16;
                    if px < area.x + area.width && py < area.y + area.height {
                        set_char(buf, px, py, BLOCK, piece_color(self.grid[row][col]), colors.bg);
                    }
                }
            }
        }

        if !self.game_over {
            let gy = ghost_y(&self.grid, &self.current.cells, self.current.x, self.current.y);
            let gc = piece_color(self.current.typ);
            for r in 0..4 {
                for col in 0..4 {
                    if self.current.cells[r][col] {
                        let bx = self.current.x + col as i32;
                        let by = gy + r as i32;
                        if bx >= 0 && bx < COLS as i32 && by >= 0 && by < ROWS as i32 {
                            set_char(buf, board_x + 1 + bx as u16, board_y + 1 + by as u16, GHOST, gc, colors.bg);
                        }
                    }
                }
            }

            let cc = piece_color(self.current.typ);
            for r in 0..4 {
                for col in 0..4 {
                    if self.current.cells[r][col] {
                        let bx = self.current.x + col as i32;
                        let by = self.current.y + r as i32;
                        if bx >= 0 && bx < COLS as i32 && by >= 0 && by < ROWS as i32 {
                            set_char(buf, board_x + 1 + bx as u16, board_y + 1 + by as u16, BLOCK, cc, colors.bg);
                        }
                    }
                }
            }
        }

        let hold_x = if board_x >= 8 { board_x - 7 } else { area.x + 1 };
        draw_text(buf, hold_x, board_y, "HOLD", colors.fg, colors.bg);
        if let Some(ht) = self.held {
            let hc = piece_color(ht);
            let hcells = get_rotated(&SHAPES[ht as usize - 1], 0);
            for r in 0..4 {
                for col in 0..4 {
                    if hcells[r][col] {
                        let sx = hold_x + col as u16;
                        let sy = board_y + 1 + r as u16;
                        if sx < area.x + area.width && sy < area.y + area.height {
                            set_char(buf, sx, sy, BLOCK, hc, colors.bg);
                        }
                    }
                }
            }
        }

        let next_x = board_x + BOARD_W + 1;
        draw_text(buf, next_x, board_y, "NEXT", colors.fg, colors.bg);
        for (i, p) in self.next_queue.iter().take(3).enumerate() {
            let nc = piece_color(p.typ);
            let ny = board_y + 1 + (i as u16) * 5;
            for r in 0..4 {
                for col in 0..4 {
                    if p.cells[r][col] {
                        let sx = next_x + col as u16;
                        let sy = ny + r as u16;
                        if sx < area.x + area.width && sy < area.y + area.height {
                            set_char(buf, sx, sy, BLOCK, nc, colors.bg);
                        }
                    }
                }
            }
        }

        let info_x = next_x;
        let info_y = board_y + 17;
        if info_y < area.y + area.height {
            draw_text(buf, info_x, info_y, &format!("SCORE: {}", self.score), colors.fg, colors.bg);
            draw_text(buf, info_x, info_y + 1, &format!("LEVEL: {}", self.level), colors.fg, colors.bg);
            draw_text(buf, info_x, info_y + 2, &format!("LINES: {}", self.lines), colors.fg, colors.bg);
        }

        if self.paused {
            let msg = "PAUSED";
            let x = board_x + (BOARD_W - msg.len() as u16) / 2;
            let y = board_y + BOARD_H / 2;
            draw_text(buf, x, y, msg, colors.warning, colors.bg);
        }

        if self.game_over {
            let msg = "GAME OVER";
            let x = board_x + (BOARD_W - msg.len() as u16) / 2;
            let y = board_y + BOARD_H / 2;
            draw_text(buf, x, y, msg, colors.error, colors.bg);
            let restart = "Press SPACE to restart";
            draw_text(buf, board_x + (BOARD_W - restart.len() as u16) / 2, y + 1, restart, colors.disabled, colors.bg);
        }
    }

    fn handle_key(&mut self, key: i32, ch: char) {
        if self.game_over {
            if key == 32 {
                self.init();
            }
            return;
        }
        if key == 27 {
            return;
        }
        if ch == 'p' || ch == 'P' {
            self.paused = !self.paused;
            return;
        }
        if self.paused {
            return;
        }
        if ch == 'c' || ch == 'C' {
            self.hold_piece();
            return;
        }
        match key {
            37 => {
                if !collides(&self.grid, &self.current.cells, self.current.x - 1, self.current.y) {
                    self.current.x -= 1;
                }
            }
            39 => {
                if !collides(&self.grid, &self.current.cells, self.current.x + 1, self.current.y) {
                    self.current.x += 1;
                }
            }
            38 => {
                let new_cells = rotated_cells(&self.current.cells);
                let offsets = [0i32, -1, 1, -2, 2];
                for &dx in &offsets {
                    if !collides(&self.grid, &new_cells, self.current.x + dx, self.current.y) {
                        self.current.cells = new_cells;
                        self.current.x += dx;
                        self.current.rotation = (self.current.rotation + 1) % 4;
                        break;
                    }
                }
            }
            40 => {
                if !collides(&self.grid, &self.current.cells, self.current.x, self.current.y + 1) {
                    self.current.y += 1;
                    self.score += 1;
                    self.drop_timer = 0.0;
                }
            }
            32 => {
                while !collides(&self.grid, &self.current.cells, self.current.x, self.current.y + 1) {
                    self.current.y += 1;
                    self.score += 2;
                }
                self.lock_and_spawn();
            }
            _ => {}
        }
    }
}
