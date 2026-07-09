use ratatui::{Frame, layout::Rect};
use crate::engine::scene::{Scene, SceneType};
use crate::engine::Engine;
use crate::engine::renderer::{draw_text, fill_rect};

static LOGO: &[&str] = &[
    "██████╗ ███████╗████████╗██████╗  ██████╗     ██╗  ██╗██╗   ██╗██████╗ ",
    "██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██╔═══██╗    ██║  ██║██║   ██║██╔══██╗",
    "██████╔╝█████╗     ██║   ██████╔╝██║   ██║    ███████║██║   ██║██████╔╝",
    "██╔══██╗██╔══╝     ██║   ██╔══██╗██║   ██║    ██╔══██║██║   ██║██╔══██╗",
    "██║  ██║███████╗   ██║   ██║  ██║╚██████╔╝    ██║  ██║╚██████╔╝██████╔╝",
    "╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝     ╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ",
];

static TAGLINE: &str = "Your Retro Gaming Hub in the Terminal";

pub struct SplashScene {
    timer: f32,
    blink: bool,
    blink_timer: f32,
}

impl SplashScene {
    pub fn new() -> Self {
        Self { timer: 0.0, blink: true, blink_timer: 0.0 }
    }
}

impl Scene for SplashScene {
    fn id(&self) -> &str { "splash" }
    fn scene_type(&self) -> SceneType { SceneType::Splash }
    fn enter(&mut self) {}

    fn update(&mut self, dt: f32) {
        self.timer += dt;
        self.blink_timer += dt;
        if self.blink_timer > 0.5 {
            self.blink = !self.blink;
            self.blink_timer = 0.0;
        }
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) {
        let c = engine.theme.colors();
        let buf = frame.buffer_mut();
        fill_rect(buf, area, c.bg);

        let logo_w = LOGO[0].len() as u16;
        let logo_h = LOGO.len() as u16;
        let start_x = (area.width.saturating_sub(logo_w)) / 2;
        let start_y = area.height.saturating_sub(logo_h) / 2 - 2;

        let pulse = (self.timer * 3.0).sin() * 0.3 + 0.7 ;

        for (i, line) in LOGO.iter().enumerate() {
            let accent = crate::engine::theme::Rgb::new(
                (122.0 * pulse) as u8,
                (162.0 * pulse) as u8,
                (247.0 * pulse) as u8,
            );
            draw_text(buf, area.x + start_x, area.y + start_y + i as u16, line, accent.to_color(), c.bg);
        }

        let tag_x = area.width.saturating_sub(TAGLINE.len() as u16) / 2;
        draw_text(buf, area.x + tag_x, area.y + start_y + logo_h + 1, TAGLINE, c.secondary, c.bg);

        let ver = "v1.0.0";
        draw_text(buf, area.x + area.width / 2 - 3, area.y + start_y + logo_h + 2, ver, c.disabled, c.bg);

        if self.blink {
            let msg = "Press any key to continue...";
            draw_text(buf, area.x + area.width / 2 - msg.len() as u16 / 2, area.y + area.height.saturating_sub(2), msg, c.disabled, c.bg);
        }
    }

    fn handle_key(&mut self, _key: i32, _ch: char) {}
}
