pub mod menu;

use ratatui::{Frame, layout::Rect};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{set_char, draw_text, fill_rect, draw_border_double};

pub struct MainMenuScene {
    pub selected: usize,
    pub sub_selected: usize,
    pub in_submenu: bool,
}

static SUBMENU: &[&str] = &["Settings", "Achievements", "Leaderboard", "Exit"];

impl MainMenuScene {
    pub fn new() -> Self { Self { selected: 0, sub_selected: 0, in_submenu: false } }

    fn render_centered_title(buf: &mut ratatui::buffer::Buffer, area: Rect, c: &crate::engine::theme::ColorScheme) {
        let title_lines = [
            "╔══════════════════════════════════════════╗",
            "║              RETRO HUB                    ║",
            "╚══════════════════════════════════════════╝",
        ];
        let title_w = title_lines[0].len() as u16;
        let title_x = (area.width.saturating_sub(title_w)) / 2;
        for (i, line) in title_lines.iter().enumerate() {
            draw_text(buf, area.x + title_x, area.y + 1 + i as u16, line, c.accent, c.bg);
        }
    }
}

impl Scene for MainMenuScene {
    fn id(&self) -> &str { "mainMenu" }
    fn scene_type(&self) -> SceneType { SceneType::MainMenu }
    fn enter(&mut self) {}
    fn exit(&mut self) {}

    fn update(&mut self, _dt: f32) {}

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let c = engine.theme.colors();
        let buf = frame.buffer_mut();
        fill_rect(buf, area, c.bg);

        Self::render_centered_title(buf, area, c);

        let games = &engine.games.games;
        let menu_start_y = 5;
        let menu_x = area.width.saturating_sub(35) / 2;
        let max_visible = area.height.saturating_sub(7) as usize;

        if !self.in_submenu {
            let total = games.len();
            let scroll_offset = if self.selected >= max_visible { self.selected - max_visible + 1 } else { 0 };

            for i in 0..max_visible.min(total) {
                let idx = scroll_offset + i;
                if idx >= total { break; }
                let (_, name) = games[idx];
                let y = area.y + menu_start_y + i as u16;
                if y >= area.height { break; }
                let sel = idx == self.selected;
                let prefix = if sel { "▶ " } else { "  " };
                let text = format!("{}{}", prefix, name);
                let fg = if sel { c.accent } else { c.fg };
                let bg = if sel { c.selection } else { c.bg };
                draw_text(buf, area.x + menu_x, y, &text, fg, bg);
            }

            if total > max_visible {
                let scroll_pct = self.selected as f32 / total.saturating_sub(1).max(1) as f32;
                let bar_y = menu_start_y + (scroll_pct * (max_visible.saturating_sub(2) as f32)) as u16;
                set_char(buf, area.x + menu_x + 34, area.y + bar_y, '█', c.accent, c.bg);
            }
        } else {
            for (i, item) in SUBMENU.iter().enumerate() {
                let y = area.y + menu_start_y + i as u16;
                if y >= area.height { break; }
                let sel = i == self.sub_selected;
                let prefix = if sel { "▶ " } else { "  " };
                let text = format!("{}{}", prefix, item);
                let fg = if sel { c.accent } else { c.fg };
                let bg = if sel { c.selection } else { c.bg };
                draw_text(buf, area.x + menu_x, y, &text, fg, bg);
            }
        }

        let hint = "↑↓ Navigate  |  Enter Select  |  Tab Menu  |  Esc Exit";
        draw_text(buf, area.x + 1, area.y + area.height.saturating_sub(1), hint, c.disabled, c.bg);
        let ver = "v1.0.0";
        draw_text(buf, area.x + area.width.saturating_sub(8), area.y + area.height.saturating_sub(1), ver, c.disabled, c.bg);
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        match key {
            9 => { self.in_submenu = !self.in_submenu; self.sub_selected = 0; }
            38 => if self.in_submenu { self.sub_selected = self.sub_selected.saturating_sub(1); }
                  else { self.selected = self.selected.saturating_sub(1); }
            40 => if self.in_submenu { self.sub_selected = (self.sub_selected + 1).min(SUBMENU.len() - 1); }
                  else { self.selected = self.selected.saturating_add(1).min(7); }
            10 => {}
            27 => {}
            _ => {}
        }
    }
}

pub struct SettingsScene {
    pub selected: usize,
    pub selected_theme_id: String,
    pub theme_names: Vec<String>,
    pub theme_ids: Vec<String>,
}

impl SettingsScene {
    pub fn new() -> Self {
        Self {
            selected: 0,
            selected_theme_id: "tokyo-night".into(),
            theme_names: vec![],
            theme_ids: vec![],
        }
    }

    pub fn refresh(&mut self, engine: &Engine) {
        self.theme_ids = engine.theme.themes.iter().map(|t| t.id.to_string()).collect();
        self.theme_names = engine.theme.themes.iter().map(|t| t.name.to_string()).collect();
        self.selected = engine.theme.current;
        self.selected_theme_id = engine.theme.themes[engine.theme.current].id.to_string();
    }
}

impl Scene for SettingsScene {
    fn id(&self) -> &str { "settings" }
    fn scene_type(&self) -> SceneType { SceneType::Settings }
    fn enter(&mut self) {}
    fn exit(&mut self) {}

    fn update(&mut self, _dt: f32) {}

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let c = engine.theme.colors();
        let buf = frame.buffer_mut();
        fill_rect(buf, area, c.bg);

        let title = "SETTINGS";
        draw_text(buf, area.x + area.width / 2 - title.len() as u16 / 2, area.y + 1, title, c.accent, c.bg);

        let panel = Rect {
            x: area.width.saturating_sub(50) / 2,
            y: 3,
            width: 50.min(area.width),
            height: (self.theme_names.len() as u16 + 4).min(area.height.saturating_sub(5)),
        };
        draw_border_double(buf, panel, c.border, c.bg);

        draw_text(buf, panel.x + 2, panel.y + 1, "Theme:", c.fg, c.bg);

        for (i, name) in self.theme_names.iter().enumerate() {
            let y = panel.y + 3 + i as u16;
            let is_current = i == engine.theme.current;
            let prefix = if i == self.selected { "▶ " } else { "  " };
            let marker = if is_current { " ✓" } else { "  " };
            let text = format!("{}{}{}", prefix, name, marker);
            let fg = if i == self.selected { c.accent } else if is_current { c.success } else { c.fg };
            let bg = if i == self.selected { c.selection } else { c.bg };
            draw_text(buf, panel.x + 4, y, &text, fg, bg);
        }

        let hint = "↑↓ Select Theme  |  Esc Back";
        draw_text(buf, area.x + area.width / 2 - hint.len() as u16 / 2, area.y + area.height.saturating_sub(1), hint, c.disabled, c.bg);
    }

    fn handle_key(&mut self, key: i32, _ch: char) {
        match key {
            38 => {
                self.selected = self.selected.saturating_sub(1);
            }
            40 => {
                self.selected = (self.selected + 1).min(self.theme_ids.len().saturating_sub(1));
            }
            10 => {
                if self.selected < self.theme_ids.len() {
                    self.selected_theme_id = self.theme_ids[self.selected].clone();
                }
            }
            _ => {}
        }
    }
}
