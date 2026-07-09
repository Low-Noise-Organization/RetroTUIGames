use std::collections::HashMap;

pub struct LeaderboardManager {
    pub scores: HashMap<String, Vec<(String, i32, u64)>>,
}

impl LeaderboardManager {
    pub fn new() -> Self { Self { scores: HashMap::new() } }
    pub fn add(&mut self, game: &str, player: &str, score: i32) {
        self.scores.entry(game.into()).or_default().push((player.into(), score, std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs()));
    }
    pub fn top(&self, game: &str, limit: usize) -> Vec<&(String, i32, u64)> {
        let mut entries: Vec<_> = self.scores.get(game).into_iter().flatten().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(limit);
        entries
    }
}
