use std::collections::HashMap;

pub struct ResourceManager {
    pub ascii_art: HashMap<String, Vec<String>>,
}

impl ResourceManager {
    pub fn new() -> Self { Self { ascii_art: HashMap::new() } }

    pub fn add_art(&mut self, name: &str, art: &str) {
        self.ascii_art.insert(name.to_string(), art.lines().map(|l| l.to_string()).collect());
    }

    pub fn get_art(&self, name: &str) -> Option<&Vec<String>> {
        self.ascii_art.get(name)
    }
}
