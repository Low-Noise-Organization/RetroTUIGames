pub mod pong;
pub mod snake;
pub mod chess;
pub mod tetris;
pub mod breakout;
pub mod minesweeper;
pub mod sudoku;
pub mod game2048;


pub struct GameRegistry {
    pub games: Vec<(&'static str, &'static str)>,
}

impl GameRegistry {
    pub fn new() -> Self {
        Self {
            games: vec![
                ("pong", "Pong"),
                ("snake", "Snake"),
                ("chess", "Chess"),
                ("tetris", "Tetris"),
                ("breakout", "Breakout"),
                ("minesweeper", "Minesweeper"),
                ("sudoku", "Sudoku"),
                ("2048", "2048"),
            ],
        }
    }
}
