use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
    pub const fn to_color(self) -> Color { Color::Rgb(self.r, self.g, self.b) }
    pub fn blend(self, other: Rgb, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            g: (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            b: (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub selection: Color,
    pub border: Color,
    pub disabled: Color,
    pub shadow: Color,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub colors: ColorScheme,
}

pub struct ThemeManager {
    pub current: usize,
    pub themes: Vec<Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self { current: 0, themes: Self::built_in() }
    }

    fn built_in() -> Vec<Theme> {
        vec![
            Theme {
                id: "tokyo-night", name: "Tokio Night",
                colors: ColorScheme {
                    bg: Rgb::new(26, 27, 38).to_color(),
                    fg: Rgb::new(169, 177, 214).to_color(),
                    accent: Rgb::new(122, 162, 247).to_color(),
                    secondary: Rgb::new(158, 206, 106).to_color(),
                    success: Rgb::new(158, 206, 106).to_color(),
                    warning: Rgb::new(224, 175, 104).to_color(),
                    error: Rgb::new(219, 75, 75).to_color(),
                    info: Rgb::new(130, 170, 255).to_color(),
                    surface: Rgb::new(36, 38, 58).to_color(),
                    surface_alt: Rgb::new(46, 48, 70).to_color(),
                    selection: Rgb::new(65, 72, 104).to_color(),
                    border: Rgb::new(55, 58, 82).to_color(),
                    disabled: Rgb::new(80, 85, 110).to_color(),
                    shadow: Rgb::new(10, 10, 16).to_color(),
                },
            },
            Theme {
                id: "catppuccin", name: "Catppuccin",
                colors: ColorScheme {
                    bg: Rgb::new(30, 30, 46).to_color(), fg: Rgb::new(205, 214, 244).to_color(),
                    accent: Rgb::new(137, 180, 250).to_color(), secondary: Rgb::new(166, 227, 161).to_color(),
                    success: Rgb::new(166, 227, 161).to_color(), warning: Rgb::new(249, 226, 175).to_color(),
                    error: Rgb::new(210, 15, 57).to_color(), info: Rgb::new(137, 180, 250).to_color(),
                    surface: Rgb::new(49, 50, 68).to_color(), surface_alt: Rgb::new(69, 71, 90).to_color(),
                    selection: Rgb::new(88, 91, 112).to_color(), border: Rgb::new(108, 112, 134).to_color(),
                    disabled: Rgb::new(127, 132, 156).to_color(), shadow: Rgb::new(15, 15, 23).to_color(),
                },
            },
            Theme {
                id: "gruvbox", name: "Gruvbox",
                colors: ColorScheme {
                    bg: Rgb::new(40, 40, 40).to_color(), fg: Rgb::new(235, 219, 178).to_color(),
                    accent: Rgb::new(184, 128, 70).to_color(), secondary: Rgb::new(152, 193, 99).to_color(),
                    success: Rgb::new(152, 193, 99).to_color(), warning: Rgb::new(214, 189, 102).to_color(),
                    error: Rgb::new(204, 74, 61).to_color(), info: Rgb::new(131, 165, 152).to_color(),
                    surface: Rgb::new(50, 50, 50).to_color(), surface_alt: Rgb::new(60, 60, 60).to_color(),
                    selection: Rgb::new(80, 73, 59).to_color(), border: Rgb::new(73, 69, 58).to_color(),
                    disabled: Rgb::new(102, 92, 84).to_color(), shadow: Rgb::new(15, 15, 15).to_color(),
                },
            },
            Theme {
                id: "nord", name: "Nord",
                colors: ColorScheme {
                    bg: Rgb::new(46, 52, 64).to_color(), fg: Rgb::new(216, 222, 233).to_color(),
                    accent: Rgb::new(136, 192, 208).to_color(), secondary: Rgb::new(163, 190, 140).to_color(),
                    success: Rgb::new(163, 190, 140).to_color(), warning: Rgb::new(235, 203, 139).to_color(),
                    error: Rgb::new(191, 97, 106).to_color(), info: Rgb::new(129, 161, 193).to_color(),
                    surface: Rgb::new(59, 66, 82).to_color(), surface_alt: Rgb::new(67, 76, 94).to_color(),
                    selection: Rgb::new(76, 86, 106).to_color(), border: Rgb::new(67, 76, 94).to_color(),
                    disabled: Rgb::new(94, 105, 125).to_color(), shadow: Rgb::new(20, 22, 28).to_color(),
                },
            },
        ]
    }

    pub fn colors(&self) -> &ColorScheme { &self.themes[self.current].colors }
    pub fn theme(&self) -> &Theme { &self.themes[self.current] }
    pub fn set_by_id(&mut self, id: &str) -> bool {
        self.themes.iter().position(|t| t.id == id).map(|i| { self.current = i; true }).unwrap_or(false)
    }
}
