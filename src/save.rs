use std::collections::HashMap;

pub struct SaveManager {
    pub saves: HashMap<String, HashMap<String, String>>,
}

impl SaveManager {
    pub fn new() -> Self { Self { saves: HashMap::new() } }
    pub fn save(&mut self, game: &str, data: HashMap<String, String>) { self.saves.insert(game.into(), data); }
    pub fn load(&self, game: &str) -> Option<&HashMap<String, String>> { self.saves.get(game) }
}
