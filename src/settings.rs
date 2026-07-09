use std::collections::HashMap;

pub struct SettingsManager {
    pub settings: HashMap<String, String>,
}

impl SettingsManager {
    pub fn new() -> Self {
        let mut s = HashMap::new();
        s.insert("theme".into(), "tokyo-night".into());
        s.insert("animations".into(), "true".into());
        s.insert("audio".into(), "true".into());
        Self { settings: s }
    }
    pub fn get(&self, key: &str) -> &str { self.settings.get(key).map(|s| s.as_str()).unwrap_or("") }
    pub fn set(&mut self, key: &str, val: &str) { self.settings.insert(key.into(), val.into()); }
}
