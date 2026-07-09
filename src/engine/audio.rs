use std::io::Write;

pub struct AudioManager {
    pub enabled: bool,
}

impl AudioManager {
    pub fn new() -> Self { Self { enabled: true } }

    pub fn beep(&self) {
        if self.enabled {
            print!("\x07");
            let _ = std::io::stdout().flush();
        }
    }
}
