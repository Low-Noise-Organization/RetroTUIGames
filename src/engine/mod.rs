pub mod theme;
pub mod scene;
pub mod events;
pub mod input;
pub mod animation;
pub mod audio;
pub mod layout;
pub mod widgets;
pub mod timing;
pub mod resources;
pub mod renderer;

use theme::ThemeManager;
pub use events::EventBus;
use animation::AnimationManager;
use audio::AudioManager;
use crate::profile::ProfileManager;
use crate::achievements::AchievementManager;
use crate::leaderboard::LeaderboardManager;
use crate::settings::SettingsManager;
use crate::save::SaveManager;
use crate::games::GameRegistry;

pub struct Engine {
    pub theme: ThemeManager,
    pub events: EventBus,
    pub animation: AnimationManager,
    pub audio: AudioManager,
    pub profile: ProfileManager,
    pub achievements: AchievementManager,
    pub leaderboard: LeaderboardManager,
    pub settings: SettingsManager,
    pub save: SaveManager,
    pub games: GameRegistry,
    pub running: bool,
    pub terminal_size: (u16, u16),
}

impl Engine {
    pub fn new() -> Self {
        Self {
            theme: ThemeManager::new(),
            events: EventBus::new(),
            animation: AnimationManager::new(),
            audio: AudioManager::new(),
            profile: ProfileManager::new(),
            achievements: AchievementManager::new(),
            leaderboard: LeaderboardManager::new(),
            settings: SettingsManager::new(),
            save: SaveManager::new(),
            games: GameRegistry::new(),
            running: true,
            terminal_size: (80, 24),
        }
    }

    pub fn stop(&mut self) { self.running = false; }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.terminal_size = (w, h);
    }
}
