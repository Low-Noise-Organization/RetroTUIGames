use std::collections::HashSet;

#[derive(Clone)]
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub points: i32,
}

pub const ALL_ACHIEVEMENTS: &[Achievement] = &[
    Achievement { id: "first_game", name: "First Steps", desc: "Play your first game", points: 10 },
    Achievement { id: "hundred_wins", name: "Century", desc: "Win 100 games", points: 50 },
    Achievement { id: "thousand_points", name: "Point Collector", desc: "Score 1000 points", points: 30 },
    Achievement { id: "ten_hours", name: "Dedicated", desc: "Play for 10 hours", points: 40 },
    Achievement { id: "pong_master", name: "Pong Master", desc: "Win 10 Pong matches", points: 30 },
    Achievement { id: "snake_king", name: "Snake King", desc: "Reach length 50 in Snake", points: 40 },
    Achievement { id: "tetris_clear", name: "Tetris Clear", desc: "Clear 100 lines in Tetris", points: 50 },
];

pub struct AchievementManager {
    pub unlocked: HashSet<String>,
}

impl AchievementManager {
    pub fn new() -> Self { Self { unlocked: HashSet::new() } }
    pub fn unlock(&mut self, id: &str) -> bool { self.unlocked.insert(id.into()) }
    pub fn is_unlocked(&self, id: &str) -> bool { self.unlocked.contains(id) }
    pub fn count(&self) -> usize { self.unlocked.len() }
    pub fn points(&self) -> i32 {
        ALL_ACHIEVEMENTS.iter().filter(|a| self.unlocked.contains(a.id)).map(|a| a.points).sum()
    }
    pub fn pct(&self) -> usize { (self.count() * 100) / ALL_ACHIEVEMENTS.len() }
}
