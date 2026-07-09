pub struct Animation {
    duration: f32,
    elapsed: f32,
    running: bool,
    repeat: bool,
    easing: Easing,
    on_update: Box<dyn FnMut(f32)>,
    on_complete: Option<Box<dyn FnMut()>>,
}

pub enum Easing {
    Linear,
    EaseOutQuad,
    EaseInOutQuad,
    EaseOutElastic,
    EaseOutBounce,
}

impl Easing {
    fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseOutQuad => t * (2.0 - t),
            Easing::EaseInOutQuad => if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t },
            Easing::EaseOutElastic => {
                if t == 0.0 || t == 1.0 { return t; }
                2.0_f32.powf(-10.0 * t) * ((t - 0.1) * 5.0 * std::f32::consts::PI).sin() + 1.0 
            }
            Easing::EaseOutBounce => {
                if t < 1.0 / 2.75 { 7.5625 * t * t }
                else if t < 2.0 / 2.75 { 7.5625 * (t - 1.5 / 2.75) * t + 0.75 }
                else if t < 2.5 / 2.75 { 7.5625 * (t - 2.25 / 2.75) * t + 0.9375 }
                else { 7.5625 * (t - 2.625 / 2.75) * t + 0.984375 }
            }
        }
    }
}

impl Animation {
    pub fn new<F: FnMut(f32) + 'static>(duration: f32, easing: Easing, on_update: F) -> Self {
        Self { duration, elapsed: 0.0, running: true, repeat: false, easing, on_update: Box::new(on_update), on_complete: None }
    }

    pub fn repeat(mut self) -> Self { self.repeat = true; self }

    pub fn on_complete<F: FnMut() + 'static>(mut self, f: F) -> Self { self.on_complete = Some(Box::new(f)); self }

    pub fn update(&mut self, dt: f32) {
        if !self.running { return; }
        self.elapsed += dt;
        let t = (self.elapsed / self.duration).min(1.0);
        (self.on_update)(self.easing.apply(t));
        if t >= 1.0 {
            if self.repeat {
                self.elapsed = 0.0;
                (self.on_update)(self.easing.apply(0.0));
            } else {
                self.running = false;
                if let Some(ref mut cb) = self.on_complete { cb(); }
            }
        }
    }

    pub fn is_running(&self) -> bool { self.running }
}

pub struct AnimationManager {
    animations: Vec<Animation>,
}

impl AnimationManager {
    pub fn new() -> Self { Self { animations: Vec::new() } }

    pub fn add(&mut self, anim: Animation) { self.animations.push(anim); }

    pub fn update(&mut self, dt: f32) {
        self.animations.retain_mut(|a| { a.update(dt); a.is_running() || a.repeat });
    }

    pub fn clear(&mut self) { self.animations.clear(); }
}
