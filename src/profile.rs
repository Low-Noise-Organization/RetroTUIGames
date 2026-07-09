use std::collections::HashMap;

pub struct Profile {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub play_time: f32,
    pub scores: HashMap<String, i32>,
    pub achievements: Vec<String>,
}

impl Profile {
    pub fn new(id: &str, name: &str) -> Self {
        Self { id: id.into(), name: name.into(), avatar: "(^_^)".into(), play_time: 0.0, scores: HashMap::new(), achievements: Vec::new() }
    }
}

pub struct ProfileManager {
    pub current: Profile,
}

impl ProfileManager {
    pub fn new() -> Self { Self { current: Profile::new("default", "Player") } }
}
