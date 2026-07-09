use ratatui::{Frame, layout::Rect};
use ratatui::style::Color;
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{self, set_char, draw_text};

const EMPTY: u8 = 0;
const PAWN: u8 = 1;
const KNIGHT: u8 = 2;
const BISHOP: u8 = 3;
const ROOK: u8 = 4;
const QUEEN: u8 = 5;
const KING: u8 = 6;
const WHITE: u8 = 8;
const BLACK: u8 = 16;

fn pt(p: u8) -> u8 { p & 0b111 }
fn pc(p: u8) -> u8 { p & 0b11000 }
fn is_c(p: u8, c: u8) -> bool { pc(p) == c }
fn mk(t: u8, c: u8) -> u8 { t | c }

fn piece_symbol(piece: u8) -> &'static str {
    match piece {
        9 => "♙", 10 => "♘", 11 => "♗", 12 => "♖", 13 => "♕", 14 => "♔",
        17 => "♟", 18 => "♞", 19 => "♝", 20 => "♜", 21 => "♛", 22 => "♚",
        _ => "·",
    }
}

fn enemy(c: u8) -> u8 { if c == WHITE { BLACK } else { WHITE } }

#[derive(Clone, Copy, Debug)]
struct ChessMove {
    from: (usize, usize),
    to: (usize, usize),
    promotion: u8,
    is_en_passant: bool,
    is_castle: bool,
}

impl ChessMove {
    fn new(from: (usize, usize), to: (usize, usize)) -> Self {
        Self { from, to, promotion: 0, is_en_passant: false, is_castle: false }
    }
}

fn to_algebraic(from: (usize, usize), to: (usize, usize), promotion: u8, is_castle: bool) -> String {
    if is_castle {
        if to.0 > from.0 { return "O-O".into(); }
        return "O-O-O".into();
    }
    let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    let ranks = ['8', '7', '6', '5', '4', '3', '2', '1'];
    let mut s = String::new();
    s.push(files[from.0]);
    s.push(ranks[from.1]);
    s.push('-');
    s.push(files[to.0]);
    s.push(ranks[to.1]);
    if promotion == QUEEN {
        s.push_str("=Q");
    }
    s
}

pub struct ChessGame {
    id: String,
    board: [[u8; 8]; 8],
    turn: u8,
    castling: [bool; 4],
    ep_square: Option<(usize, usize)>,
    half_move: i32,
    full_move: i32,
    history: Vec<String>,
    position_hashes: Vec<u64>,
    cursor_x: usize,
    cursor_y: usize,
    sel_x: usize,
    sel_y: usize,
    selected: bool,
    game_over: bool,
    message: String,
    width: u16,
    height: u16,
    last_move: Option<(usize, usize, usize, usize)>,
    needs_ai: bool,
}

fn hash_position(board: &[[u8; 8]; 8], turn: u8, castling: &[bool; 4], ep: Option<(usize, usize)>) -> u64 {
    let mut h: u64 = 0x12345678;
    for y in 0..8 {
        for x in 0..8 {
            let p = board[y][x];
            if p != EMPTY {
                h = h.wrapping_mul(0x9E3779B97F4A7C15u64).wrapping_add(p as u64);
            }
        }
    }
    h = h.wrapping_mul(0x9E3779B97F4A7C15u64).wrapping_add(if turn == WHITE { 1 } else { 2 });
    for &c in castling {
        h = h.wrapping_mul(0x9E3779B97F4A7C15u64).wrapping_add(if c { 3 } else { 5 });
    }
    if let Some((ex, ey)) = ep {
        h = h.wrapping_mul(0x9E3779B97F4A7C15u64).wrapping_add(7 + ex as u64 * 8 + ey as u64);
    }
    h
}

impl ChessGame {
    pub fn new() -> Self {
        Self {
            id: "chess".into(),
            board: [[EMPTY; 8]; 8],
            turn: WHITE,
            castling: [false; 4],
            ep_square: None,
            half_move: 0,
            full_move: 1,
            history: Vec::new(),
            position_hashes: Vec::new(),
            cursor_x: 4,
            cursor_y: 7,
            sel_x: 0,
            sel_y: 0,
            selected: false,
            game_over: false,
            message: String::new(),
            width: 80,
            height: 24,
            last_move: None,
            needs_ai: false,
        }
    }

    fn reset_state(&mut self) {
        self.board = Self::initial_board();
        self.turn = WHITE;
        self.castling = [true, true, true, true];
        self.ep_square = None;
        self.half_move = 0;
        self.full_move = 1;
        self.history.clear();
        self.position_hashes.clear();
        self.cursor_x = 4;
        self.cursor_y = 7;
        self.selected = false;
        self.game_over = false;
        self.message = String::new();
        self.last_move = None;
        self.needs_ai = false;
        self.position_hashes.push(hash_position(&self.board, self.turn, &self.castling, self.ep_square));
    }

    fn initial_board() -> [[u8; 8]; 8] {
        let mut b = [[EMPTY; 8]; 8];
        let back_rank = [ROOK, KNIGHT, BISHOP, QUEEN, KING, BISHOP, KNIGHT, ROOK];
        for x in 0..8 {
            b[0][x] = mk(back_rank[x], BLACK);
            b[1][x] = mk(PAWN, BLACK);
            b[6][x] = mk(PAWN, WHITE);
            b[7][x] = mk(back_rank[x], WHITE);
        }
        b
    }

    fn find_king(&self, color: u8) -> Option<(usize, usize)> {
        for y in 0..8 {
            for x in 0..8 {
                if pt(self.board[y][x]) == KING && is_c(self.board[y][x], color) {
                    return Some((x, y));
                }
            }
        }
        None
    }

    fn is_attacked_by(&self, square: (usize, usize), attacker: u8) -> bool {
        let (sx, sy) = (square.0 as i32, square.1 as i32);

        let pawn_dir: i32 = if attacker == WHITE { -1 } else { 1 };
        for dx in [-1, 1] {
            let nx = sx + dx;
            let ny = sy + pawn_dir;
            if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                let p = self.board[ny as usize][nx as usize];
                if pt(p) == PAWN && is_c(p, attacker) { return true; }
            }
        }

        for &(dx, dy) in &[(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)] {
            let nx = sx + dx;
            let ny = sy + dy;
            if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                let p = self.board[ny as usize][nx as usize];
                if pt(p) == KNIGHT && is_c(p, attacker) { return true; }
            }
        }

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = sx + dx;
                let ny = sy + dy;
                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                    let p = self.board[ny as usize][nx as usize];
                    if pt(p) == KING && is_c(p, attacker) { return true; }
                }
            }
        }

        for &(dx, dy) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
            let mut nx = sx + dx;
            let mut ny = sy + dy;
            while nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                let p = self.board[ny as usize][nx as usize];
                if p != EMPTY {
                    if is_c(p, attacker) && (pt(p) == BISHOP || pt(p) == QUEEN) { return true; }
                    break;
                }
                nx += dx;
                ny += dy;
            }
        }

        for &(dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let mut nx = sx + dx;
            let mut ny = sy + dy;
            while nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                let p = self.board[ny as usize][nx as usize];
                if p != EMPTY {
                    if is_c(p, attacker) && (pt(p) == ROOK || pt(p) == QUEEN) { return true; }
                    break;
                }
                nx += dx;
                ny += dy;
            }
        }

        false
    }

    fn is_in_check(&self, color: u8) -> bool {
        if let Some(king_pos) = self.find_king(color) {
            return self.is_attacked_by(king_pos, enemy(color));
        }
        false
    }

    fn generate_pseudo_moves(&self, color: u8) -> Vec<ChessMove> {
        let mut moves = Vec::new();
        let enemy_color = enemy(color);
        let forward: i32 = if color == WHITE { -1 } else { 1 };
        let start_row: usize = if color == WHITE { 6 } else { 1 };
        let promo_row: usize = if color == WHITE { 0 } else { 7 };

        for y in 0..8 {
            for x in 0..8 {
                let p = self.board[y][x];
                if p == EMPTY || !is_c(p, color) { continue; }

                match pt(p) {
                    PAWN => {
                        let ny = y as i32 + forward;
                        if ny >= 0 && ny < 8 {
                            if self.board[ny as usize][x] == EMPTY {
                                let to = (x, ny as usize);
                                if ny as usize == promo_row {
                                    for promo in [QUEEN, KNIGHT, BISHOP, ROOK] {
                                        moves.push(ChessMove { from: (x, y), to, promotion: promo, ..ChessMove::new((x, y), to) });
                                    }
                                } else {
                                    moves.push(ChessMove::new((x, y), to));
                                }

                                if y == start_row {
                                    let ny2 = y as i32 + 2 * forward;
                                    let mid_y = y as i32 + forward;
                                    if self.board[mid_y as usize][x] == EMPTY && self.board[ny2 as usize][x] == EMPTY {
                                        moves.push(ChessMove::new((x, y), (x, ny2 as usize)));
                                    }
                                }
                            }

                            for dx in [-1, 1] {
                                let nx = x as i32 + dx;
                                if nx >= 0 && nx < 8 {
                                    let target = self.board[ny as usize][nx as usize];
                                    if target != EMPTY && is_c(target, enemy_color) {
                                        let to = (nx as usize, ny as usize);
                                        if ny as usize == promo_row {
                                            for promo in [QUEEN, KNIGHT, BISHOP, ROOK] {
                                                moves.push(ChessMove { from: (x, y), to, promotion: promo, ..ChessMove::new((x, y), to) });
                                            }
                                        } else {
                                            moves.push(ChessMove::new((x, y), to));
                                        }
                                    }

                                    if let Some((epx, epy)) = self.ep_square {
                                        if epx == nx as usize && epy == ny as usize {
                                            let to = (nx as usize, ny as usize);
                                            moves.push(ChessMove { from: (x, y), to, is_en_passant: true, ..ChessMove::new((x, y), to) });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    KNIGHT => {
                        for &(dx, dy) in &[(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)] {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                                let target = self.board[ny as usize][nx as usize];
                                if target == EMPTY || is_c(target, enemy_color) {
                                    moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                }
                            }
                        }
                    }

                    BISHOP => {
                        for &(dx, dy) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                            let mut nx = x as i32 + dx;
                            let mut ny = y as i32 + dy;
                            while nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                                let target = self.board[ny as usize][nx as usize];
                                if target == EMPTY {
                                    moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                } else {
                                    if is_c(target, enemy_color) {
                                        moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                    }
                                    break;
                                }
                                nx += dx;
                                ny += dy;
                            }
                        }
                    }

                    ROOK => {
                        for &(dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            let mut nx = x as i32 + dx;
                            let mut ny = y as i32 + dy;
                            while nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                                let target = self.board[ny as usize][nx as usize];
                                if target == EMPTY {
                                    moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                } else {
                                    if is_c(target, enemy_color) {
                                        moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                    }
                                    break;
                                }
                                nx += dx;
                                ny += dy;
                            }
                        }
                    }

                    QUEEN => {
                        for &(dx, dy) in &[(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
                            let mut nx = x as i32 + dx;
                            let mut ny = y as i32 + dy;
                            while nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                                let target = self.board[ny as usize][nx as usize];
                                if target == EMPTY {
                                    moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                } else {
                                    if is_c(target, enemy_color) {
                                        moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                    }
                                    break;
                                }
                                nx += dx;
                                ny += dy;
                            }
                        }
                    }

                    KING => {
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 { continue; }
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                                    let target = self.board[ny as usize][nx as usize];
                                    if target == EMPTY || is_c(target, enemy_color) {
                                        moves.push(ChessMove::new((x, y), (nx as usize, ny as usize)));
                                    }
                                }
                            }
                        }

                        let (rk, king_side_idx, queen_side_idx) = if color == WHITE {
                            (7usize, 0usize, 1usize)
                        } else {
                            (0usize, 2usize, 3usize)
                        };
                        if y == rk && x == 4 {
                            if self.castling[king_side_idx] && self.board[rk][5] == EMPTY && self.board[rk][6] == EMPTY && self.board[rk][7] == mk(ROOK, color) {
                                let mut clear = true;
                                for col in 4..=6 {
                                    if self.is_attacked_by((col, rk), enemy_color) {
                                        clear = false;
                                        break;
                                    }
                                }
                                if clear {
                                    moves.push(ChessMove { from: (x, y), to: (6, rk), is_castle: true, ..ChessMove::new((x, y), (6, rk)) });
                                }
                            }
                            if self.castling[queen_side_idx] && self.board[rk][3] == EMPTY && self.board[rk][2] == EMPTY && self.board[rk][1] == EMPTY && self.board[rk][0] == mk(ROOK, color) {
                                let mut clear = true;
                                for col in 2..=4 {
                                    if self.is_attacked_by((col, rk), enemy_color) {
                                        clear = false;
                                        break;
                                    }
                                }
                                if clear {
                                    moves.push(ChessMove { from: (x, y), to: (2, rk), is_castle: true, ..ChessMove::new((x, y), (2, rk)) });
                                }
                            }
                        }
                    }

                    _ => {}
                }
            }
        }

        moves
    }

    fn apply_move(&self, m: &ChessMove) -> ([[u8; 8]; 8], [bool; 4], Option<(usize, usize)>, u8) {
        let mut b = self.board;
        let mut cast = self.castling;
        let captured = b[m.to.1][m.to.0];

        b[m.to.1][m.to.0] = b[m.from.1][m.from.0];
        b[m.from.1][m.from.0] = EMPTY;

        let mut new_ep = None;

        if m.is_en_passant {
            let cap_y = m.from.1;
            b[cap_y][m.to.0] = EMPTY;
        }

        if pt(b[m.to.1][m.to.0]) == PAWN && (m.to.1 as i32 - m.from.1 as i32).abs() == 2 {
            let ep_y = (m.from.1 + m.to.1) / 2;
            new_ep = Some((m.from.0, ep_y));
        }

        if m.promotion != 0 {
            let color = pc(b[m.to.1][m.to.0]);
            b[m.to.1][m.to.0] = mk(m.promotion, color);
        }

        if m.is_castle {
            let row = m.from.1;
            if m.to.0 == 6 {
                b[row][5] = b[row][7];
                b[row][7] = EMPTY;
            } else {
                b[row][3] = b[row][0];
                b[row][0] = EMPTY;
            }
        }

        if pt(self.board[m.from.1][m.from.0]) == KING {
            if is_c(self.board[m.from.1][m.from.0], WHITE) {
                cast[0] = false;
                cast[1] = false;
            } else {
                cast[2] = false;
                cast[3] = false;
            }
        }

        for cy in 0..8 {
            for cx in 0..8 {
                if pt(b[cy][cx]) == ROOK {
                    if is_c(b[cy][cx], WHITE) {
                        if cx == 7 && cy == 7 { cast[0] = false; }
                        if cx == 0 && cy == 7 { cast[1] = false; }
                    } else {
                        if cx == 7 && cy == 0 { cast[2] = false; }
                        if cx == 0 && cy == 0 { cast[3] = false; }
                    }
                }
            }
        }
        if m.from == (7, 7) || m.to == (7, 7) { cast[0] = false; }
        if m.from == (0, 7) || m.to == (0, 7) { cast[1] = false; }
        if m.from == (7, 0) || m.to == (7, 0) { cast[2] = false; }
        if m.from == (0, 0) || m.to == (0, 0) { cast[3] = false; }

        (b, cast, new_ep, captured)
    }

    fn generate_legal_moves(&self, color: u8) -> Vec<ChessMove> {
        let pseudo = self.generate_pseudo_moves(color);
        let mut legal = Vec::new();

        for m in pseudo {
            let (new_board, new_castling, new_ep, _) = self.apply_move(&m);

            let our_color = color;
            let mut temp_game = ChessGame::new();
            temp_game.board = new_board;
            temp_game.castling = new_castling;
            temp_game.ep_square = new_ep;

            if !temp_game.is_in_check(our_color) {
                legal.push(m);
            }
        }

        legal
    }

    fn make_move_and_advance(&mut self, m: &ChessMove) {
        let moving_piece = self.board[m.from.1][m.from.0];
        let captured = self.board[m.to.1][m.to.0];

        let (new_board, new_castling, new_ep, _) = self.apply_move(m);
        self.board = new_board;
        self.castling = new_castling;
        self.ep_square = new_ep;

        if captured != EMPTY || pt(moving_piece) == PAWN {
            self.half_move = 0;
        } else {
            self.half_move += 1;
        }

        self.turn = enemy(self.turn);
        if self.turn == WHITE {
            self.full_move += 1;
        }

        let notation = to_algebraic(m.from, m.to, m.promotion, m.is_castle);
        self.history.push(notation);
        self.last_move = Some((m.from.0, m.from.1, m.to.0, m.to.1));

        let h = hash_position(&self.board, self.turn, &self.castling, self.ep_square);
        self.position_hashes.push(h);

        self.update_game_state();
    }

    fn update_game_state(&mut self) {
        let legal_moves = self.generate_legal_moves(self.turn);
        let in_check = self.is_in_check(self.turn);

        if legal_moves.is_empty() {
            self.game_over = true;
            if in_check {
                let winner = if self.turn == WHITE { "Black" } else { "White" };
                self.message = format!("Checkmate! {} wins!", winner);
            } else {
                self.message = "Stalemate!".into();
            }
            return;
        }

        if in_check {
            self.message = "Check!".into();
        } else {
            self.message = String::new();
        }

        if self.half_move >= 100 {
            self.game_over = true;
            self.message = "Draw - 50 move rule".into();
            return;
        }

        let mut count = 0;
        let current = self.position_hashes[self.position_hashes.len() - 1];
        for &h in &self.position_hashes {
            if h == current {
                count += 1;
            }
        }
        if count >= 3 {
            self.game_over = true;
            self.message = "Draw - Threefold repetition".into();
            return;
        }

        let mut pieces = 0usize;
        let mut minor = false;
        for cy in 0..8 {
            for cx in 0..8 {
                if self.board[cy][cx] != EMPTY {
                    pieces += 1;
                    let t = pt(self.board[cy][cx]);
                    if t == BISHOP || t == KNIGHT { minor = true; }
                }
            }
        }
        if pieces == 2 || (pieces == 3 && minor) {
            self.game_over = true;
            self.message = "Draw - Insufficient material".into();
        }
    }

    fn evaluate(&self) -> i32 {
        let mut score = 0i32;

        let pawn_table: [i32; 64] = [
            0,  0,  0,  0,  0,  0,  0,  0,
            50, 50, 50, 50, 50, 50, 50, 50,
            10, 10, 20, 30, 30, 20, 10, 10,
            5,  5, 10, 25, 25, 10,  5,  5,
            0,  0,  0, 20, 20,  0,  0,  0,
            5, -5,-10,  0,  0,-10, -5,  5,
            5, 10, 10,-20,-20, 10, 10,  5,
            0,  0,  0,  0,  0,  0,  0,  0,
        ];
        let knight_table: [i32; 64] = [
            -50,-40,-30,-30,-30,-30,-40,-50,
            -40,-20,  0,  0,  0,  0,-20,-40,
            -30,  0, 10, 15, 15, 10,  0,-30,
            -30,  5, 15, 20, 20, 15,  5,-30,
            -30,  0, 15, 20, 20, 15,  0,-30,
            -30,  5, 10, 15, 15, 10,  5,-30,
            -40,-20,  0,  5,  5,  0,-20,-40,
            -50,-40,-30,-30,-30,-30,-40,-50,
        ];
        let bishop_table: [i32; 64] = [
            -20,-10,-10,-10,-10,-10,-10,-20,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -10,  0, 10, 10, 10, 10,  0,-10,
            -10,  5,  5, 10, 10,  5,  5,-10,
            -10,  0,  5, 10, 10,  5,  0,-10,
            -10, 10, 10, 10, 10, 10, 10,-10,
            -10,  5,  0,  0,  0,  0,  5,-10,
            -20,-10,-10,-10,-10,-10,-10,-20,
        ];
        let rook_table: [i32; 64] = [
            0,  0,  0,  0,  0,  0,  0,  0,
            5, 10, 10, 10, 10, 10, 10,  5,
           -5,  0,  0,  0,  0,  0,  0, -5,
           -5,  0,  0,  0,  0,  0,  0, -5,
           -5,  0,  0,  0,  0,  0,  0, -5,
           -5,  0,  0,  0,  0,  0,  0, -5,
           -5,  0,  0,  0,  0,  0,  0, -5,
            0,  0,  0,  5,  5,  0,  0,  0,
        ];
        let queen_table: [i32; 64] = [
            -20,-10,-10, -5, -5,-10,-10,-20,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -10,  0,  5,  5,  5,  5,  0,-10,
             -5,  0,  5,  5,  5,  5,  0, -5,
              0,  0,  5,  5,  5,  5,  0, -5,
            -10,  5,  5,  5,  5,  5,  0,-10,
            -10,  0,  5,  0,  0,  0,  0,-10,
            -20,-10,-10, -5, -5,-10,-10,-20,
        ];
        let king_table: [i32; 64] = [
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -20,-30,-30,-40,-40,-30,-30,-20,
            -10,-20,-20,-20,-20,-20,-20,-10,
             20, 20,  0,  0,  0,  0, 20, 20,
             20, 30, 10,  0,  0, 10, 30, 20,
        ];

        for y in 0..8 {
            for x in 0..8 {
                let p = self.board[y][x];
                if p == EMPTY { continue; }
                let piece_type = pt(p);
                let color = pc(p);
                let idx = if color == WHITE { (7 - y) * 8 + x } else { y * 8 + (7 - x) };

                let (value, table): (i32, &[i32; 64]) = match piece_type {
                    PAWN => (100, &pawn_table),
                    KNIGHT => (320, &knight_table),
                    BISHOP => (330, &bishop_table),
                    ROOK => (500, &rook_table),
                    QUEEN => (900, &queen_table),
                    KING => (20000, &king_table),
                    _ => continue,
                };

                let sign = if color == WHITE { 1 } else { -1 };
                score += sign * (value + table[idx]);
            }
        }

        let white_moves = self.generate_legal_moves(WHITE).len() as i32;
        let black_moves = self.generate_legal_moves(BLACK).len() as i32;
        score += (white_moves - black_moves) * 2;

        score
    }

    fn minimax(&self, depth: i32, mut alpha: i32, mut beta: i32, is_maximizing: bool) -> (i32, Option<ChessMove>) {
        if depth == 0 {
            return (self.evaluate(), None);
        }

        let color = if is_maximizing { WHITE } else { BLACK };
        let moves = self.generate_legal_moves(color);

        if moves.is_empty() {
            let in_check = self.is_in_check(color);
            if in_check {
                return (if is_maximizing { -99999 - depth } else { 99999 + depth }, None);
            }
            return (0, None);
        }

        let mut best_move: Option<ChessMove> = None;

        let mut scored_moves: Vec<(i32, &ChessMove)> = moves.iter().map(|m| {
            let mut s = 0i32;
            let target = self.board[m.to.1][m.to.0];
            if target != EMPTY {
                let victim = match pt(target) {
                    QUEEN => 900,
                    ROOK => 500,
                    BISHOP => 330,
                    KNIGHT => 320,
                    PAWN => 100,
                    _ => 0,
                };
                let attacker = match pt(self.board[m.from.1][m.from.0]) {
                    PAWN => 100,
                    KNIGHT => 320,
                    BISHOP => 330,
                    ROOK => 500,
                    QUEEN => 900,
                    _ => 0,
                };
                s = victim - attacker;
            }
            if m.promotion != 0 { s += 800; }
            (s, m)
        }).collect();

        scored_moves.sort_by(|a, b| b.0.cmp(&a.0));

        if is_maximizing {
            let mut max_eval = i32::MIN + 100000;
            for (_, m) in &scored_moves {
                let (new_board, new_castling, new_ep, _) = self.apply_move(m);
                let mut temp = ChessGame::new();
                temp.board = new_board;
                temp.castling = new_castling;
                temp.ep_square = new_ep;

                let (eval, _) = temp.minimax(depth - 1, alpha, beta, false);
                if eval > max_eval {
                    max_eval = eval;
                    best_move = Some(**m);
                }
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            }
            (max_eval, best_move)
        } else {
            let mut min_eval = i32::MAX - 100000;
            for (_, m) in &scored_moves {
                let (new_board, new_castling, new_ep, _) = self.apply_move(m);
                let mut temp = ChessGame::new();
                temp.board = new_board;
                temp.castling = new_castling;
                temp.ep_square = new_ep;

                let (eval, _) = temp.minimax(depth - 1, alpha, beta, true);
                if eval < min_eval {
                    min_eval = eval;
                    best_move = Some(**m);
                }
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
            (min_eval, best_move)
        }
    }

    fn ai_move(&mut self) {
        if self.game_over { return; }
        let (_, best_move) = self.minimax(3, i32::MIN + 100000, i32::MAX - 100000, false);
        if let Some(m) = best_move {
            self.make_move_and_advance(&m);
        }
    }
}

impl Scene for ChessGame {
    fn id(&self) -> &str { &self.id }
    fn scene_type(&self) -> SceneType { SceneType::Game }

    fn set_terminal_size(&mut self, w: u16, h: u16) {
        self.width = w.max(30);
        self.height = h.max(15);
    }

    fn init(&mut self) {
        self.reset_state();
    }

    fn enter(&mut self) {}

    fn update(&mut self, dt: f32) {
        if self.needs_ai && !self.game_over && self.turn == BLACK {
            self.needs_ai = false;
            if dt < 0.5 {
                self.ai_move();
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let c = engine.theme.colors();
        let buf = frame.buffer_mut();

        renderer::fill_rect(buf, area, c.bg);

        let board_width: i32 = 19;
        let board_height: i32 = 9;
        let avail_w = area.width.min(self.width);
        let avail_h = area.height.min(self.height);
        let bx = area.x as i32 + (avail_w as i32 - board_width) / 2;
        let by = area.y as i32 + (avail_h as i32 - board_height) / 2;

        let light_sq = Color::Rgb(240, 217, 181);
        let dark_sq = Color::Rgb(181, 136, 99);
        let piece_white = Color::White;
        let piece_black = Color::Black;

        for y in 0..8 {
            for x in 0..8 {
                let sx = (bx + 2 + (x as i32) * 2) as u16;
                let sy = (by + y as i32) as u16;
                let is_light = (x + y) % 2 == 0;
                let mut bg = if is_light { light_sq } else { dark_sq };

                if let Some((fx, fy, tx, ty)) = self.last_move {
                    if x == fx && y == fy {
                        bg = Color::Rgb(205, 180, 100);
                    }
                    if x == tx && y == ty {
                        bg = Color::Rgb(205, 180, 100);
                    }
                }

                if x == self.cursor_x && y == self.cursor_y {
                    bg = Color::Rgb(100, 180, 255);
                }

                if self.selected && x == self.sel_x && y == self.sel_y {
                    bg = Color::Rgb(255, 220, 80);
                }

                if self.selected {
                    let p = self.board[self.sel_y][self.sel_x];
                    if p != EMPTY && is_c(p, self.turn) {
                        let legal = self.generate_legal_moves(self.turn);
                        let can_move_to = legal.iter().any(|lm| lm.to == (x, y));
                        if can_move_to {
                            if self.board[y][x] != EMPTY {
                                bg = Color::Rgb(200, 80, 80);
                            }
                        }
                    }
                }

                let piece = self.board[y][x];
                let fg = if piece == EMPTY {
                    bg
                } else if is_c(piece, WHITE) {
                    piece_white
                } else {
                    piece_black
                };

                set_char(buf, sx, sy, ' ', c.bg, bg);
                set_char(buf, sx + 1, sy, ' ', c.bg, bg);

                if piece != EMPTY {
                    let sym = piece_symbol(piece).chars().next().unwrap_or('·');
                    set_char(buf, sx, sy, sym, fg, bg);
                } else if self.selected {
                    let legal = self.generate_legal_moves(self.turn);
                    if legal.iter().any(|lm| lm.to == (x, y)) {
                        set_char(buf, sx, sy, '◉', Color::Rgb(80, 80, 80), bg);
                    }
                }
            }
        }

        let file_chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        for y in 0..8 {
            let rank = (8 - y).to_string();
            let label_y = (by + y as i32) as u16;
            draw_text(buf, (bx - 2) as u16, label_y, &format!("{} ", rank), c.fg, c.bg);
            draw_text(buf, (bx + 2 + 8 * 2 + 1) as u16, label_y, &format!(" {}", rank), c.fg, c.bg);
        }

        for x in 0..8 {
            let label_x = (bx + 2 + (x as i32) * 2) as u16;
            draw_text(buf, label_x, (by + 8) as u16, &file_chars[x].to_string(), c.fg, c.bg);
        }

        let turn_text = if self.turn == WHITE { "White's turn" } else { "Black's turn (AI)" };
        draw_text(buf, (bx + 2) as u16, (by - 2) as u16, turn_text, c.accent, c.bg);

        if !self.message.is_empty() {
            let msg_color = if self.game_over { c.error } else { c.warning };
            draw_text(buf, (bx + 2) as u16, (by - 1) as u16, &self.message, msg_color, c.bg);
        }

        let panel_x = (bx + board_width + 3) as u16;
        let panel_y = by as u16;
        let max_visible = (area.y + avail_h).saturating_sub(panel_y + 1) as usize;

        draw_text(buf, panel_x, panel_y, "── Moves ──", c.border, c.bg);
        let history_start = if self.history.len() > max_visible {
            self.history.len() - max_visible
        } else {
            0
        };

        for (i, h) in self.history.iter().enumerate().skip(history_start) {
            let idx = i - history_start;
            let label = if i % 2 == 0 {
                format!("{}.{} {}", (i / 2) + 1, "", h)
            } else {
                format!("   ... {}", h)
            };
            let draw_y = panel_y + 1 + idx as u16;
            if draw_y < area.y + avail_h.saturating_sub(1) {
                draw_text(buf, panel_x, draw_y, &label, c.fg, c.bg);
            } else {
                break;
            }
        }

        let controls_y = (by + board_height + 1) as u16;
        if controls_y < area.y + avail_h {
            draw_text(buf, (bx + 2) as u16, controls_y, "Arrows: Move cursor | Enter: Select/Move | ESC: Exit", c.disabled, c.bg);
        }
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        if self.game_over {
            return;
        }

        match key {
            38 => {
                if self.cursor_y > 0 { self.cursor_y -= 1; }
            }
            40 => {
                if self.cursor_y < 7 { self.cursor_y += 1; }
            }
            37 => {
                if self.cursor_x > 0 { self.cursor_x -= 1; }
            }
            39 => {
                if self.cursor_x < 7 { self.cursor_x += 1; }
            }
            10 | 13 => {
                if self.turn != WHITE || self.game_over { return; }

                if !self.selected {
                    let p = self.board[self.cursor_y][self.cursor_x];
                    if p != EMPTY && is_c(p, WHITE) {
                        self.sel_x = self.cursor_x;
                        self.sel_y = self.cursor_y;
                        self.selected = true;
                    }
                } else {
                    if self.cursor_x == self.sel_x && self.cursor_y == self.sel_y {
                        self.selected = false;
                        return;
                    }

                    let m = ChessMove::new((self.sel_x, self.sel_y), (self.cursor_x, self.cursor_y));
                    let legal = self.generate_legal_moves(WHITE);

                    let matched = legal.iter().find(|lm| lm.from == m.from && lm.to == m.to);
                    if let Some(&found) = matched {
                        self.selected = false;
                        self.make_move_and_advance(&found);
                        if !self.game_over && self.turn == BLACK {
                            self.needs_ai = true;
                        }
                    } else {
                        let p = self.board[self.cursor_y][self.cursor_x];
                        if p != EMPTY && is_c(p, WHITE) {
                            self.sel_x = self.cursor_x;
                            self.sel_y = self.cursor_y;
                        } else {
                            self.selected = false;
                        }
                    }
                }
            }
            27 => {}
            _ => {}
        }
    }
}
