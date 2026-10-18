use ev3dev_lang_rust::{Button, Led};

#[derive(Debug, Clone)]
/// Holds UI elements
pub struct UI {
    pub led: Led,
    pub button: Button,
}

impl Default for UI {
    /// See [UI::new](UI::new)
    fn default() -> Self {
        Self::new()
    }
}

impl UI {
    /// Returns new [UI](UI) object with `Leds` and `Buttons`
    pub fn new() -> Self {
        Self {
            led: Led::new().unwrap(),
            button: Button::new().unwrap(),
        }
    }
}
