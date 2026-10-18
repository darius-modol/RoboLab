use serde::{Deserialize, Serialize};
use std::fs;

/// Holds reference values for colors </br>
/// Can be used to detect if a color matches another: </br>
/// ```
/// let colors = Colors::new();
/// let is_red = colors.is_red(r,g,b)
/// let is_blue = colors.is_blue(r,g,b)
/// let is_black = colors.is_black(r,g,b)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    white: (i32, i32, i32),
    black: (i32, i32, i32),
    red: (i32, i32, i32),
    blue: (i32, i32, i32),
}

impl Colors {
    /// Return new Colors object with default values </br>
    /// set reference colors by using
    /// ```
    /// let colors = Colors::new();
    /// colors.set_white(255,255,255);
    /// colors.set_black(0,0,0);
    /// colors.set_red(255,0,0);
    /// colors.set_blue(255,0,0);
    /// ```
    pub fn new() -> Self {
        Self {
            white: (255, 255, 255),
            black: (0, 0, 0),
            red: (255, 0, 0),
            blue: (0, 0, 255),
        }
    }

    /// Calculates the brightness for a color given in `(r, g, b)` </br>
    /// the brightness is based on white and black values set by [self.set_white()](self.set_white) and [self.set_black()](self.set_black)
    /// where white has brightness `100` and black `0`
    /// - returns `brightness` in `(0..=100)`
    /// # Examples
    /// ```
    /// let colors = Colors::new();
    /// let b = colors.get_color_brightness(255,255,255);
    /// assert_eq!(b, 1)
    /// ```
    pub fn get_color_brightness(&self, r: i32, g: i32, b: i32) -> i32 {
        let black = self.black;
        let white = self.white;

        let r = (r - black.0) * 100 / (white.0 - black.0);
        let g = (g - black.1) * 100 / (white.1 - black.1);
        let b = (b - black.2) * 100 / (white.2 - black.2);

        let brightness_raw = (r + g + b) / 3;
        brightness_raw.clamp(0, 100)
    }

    /// Returns [Colors](Colors) object deserialized from `calibration.json`
    pub fn load_from_file() -> Option<Self> {
        let file = fs::File::open("calibration.json").ok()?;
        serde_json::from_reader(file).ok()
    }

    /// Serializes this [Colors](Colors) object to JSON and writes it to `calibration.json`
    pub fn save_to_file(&self) {
        let file = fs::File::create("calibration.json").unwrap();
        serde_json::to_writer(file, self).unwrap();
    }

    /// Checks if the given value roughly matches black
    pub fn is_black(&self, r: i32, g: i32, b: i32) -> bool {
        (r - self.black.0).abs() < 25
            && (g - self.black.1).abs() < 25
            && (b - self.black.2).abs() < 25
    }

    /// Checks if the given value roughly matches blue
    pub fn is_blue(&self, r: i32, g: i32, b: i32) -> bool {
        (r - self.blue.0).abs() < 15 && (g - self.blue.1).abs() < 15 && (b - self.blue.2).abs() < 15
    }

    /// Checks if the given value roughly matches red
    pub fn is_red(&self, r: i32, g: i32, b: i32) -> bool {
        (r - self.red.0).abs() < 15 && (g - self.red.1).abs() < 15 && (b - self.red.2).abs() < 15
    }

    pub fn white(&self) -> (i32, i32, i32) {
        self.white
    }

    pub fn black(&self) -> (i32, i32, i32) {
        self.black
    }

    pub fn red(&self) -> (i32, i32, i32) {
        self.red
    }

    pub fn blue(&self) -> (i32, i32, i32) {
        self.blue
    }

    pub fn set_white(&mut self, white: (i32, i32, i32)) {
        self.white = white;
    }

    pub fn set_black(&mut self, black: (i32, i32, i32)) {
        self.black = black;
    }

    pub fn set_red(&mut self, red: (i32, i32, i32)) {
        self.red = red;
    }

    pub fn set_blue(&mut self, blue: (i32, i32, i32)) {
        self.blue = blue;
    }
}

impl Default for Colors {
    /// See [Colors::new](Colors::new)
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Test that clean white color gives brightness of 1.0 -> based on defaults, should be the same when calibrated
    #[test]
    fn test_brightness_white_is_one() {
        let colors = Colors::new();
        let b = colors.get_color_brightness(255, 255, 255);
        assert_eq!((b - 100).abs(), 0, "Expected brightness to be 100, got {b}");
    }

    /// Test that clean black color gives brightness of 0.0 -> based on defaults, should be the same when calibrated
    #[test]
    fn test_brightness_black_is_zero() {
        let colors = Colors::new();
        let b = colors.get_color_brightness(0, 0, 0);
        assert_eq!(b.abs(), 0, "Expected brightness to be 0, got {b}");
    }

    /// Anything above the calibrated white (defaults) should clamp to 1.0
    #[test]
    fn test_brightness_clamps_above_white() {
        let colors = Colors::new();
        let b = colors.get_color_brightness(300, 300, 300);
        assert_eq!((b - 100).abs(), 0, "Should clamp to 100, got {b}");
    }

    /// Anything below the calibrated black (10,10,10) should clamp to 0.0
    #[test]
    fn test_brightness_clamps_below_black() {
        let mut colors = Colors::new();
        colors.set_black((10, 10, 10));
        let b = colors.get_color_brightness(0, 0, 0);
        assert_eq!(b.abs(), 0, "Should clamp to 0, got {b}");
    }

    /// Test that clean gray gives a brightness of around 0.5
    #[test]
    fn test_brightness_midpoint() {
        let colors = Colors::new();
        let b = colors.get_color_brightness(128, 128, 128);
        assert_eq!(b.abs(), 50, "Expected brightness to be around 50, got {b}");
    }

    /// Test writing calibration to file and reading it back gives the same values
    #[test]
    fn test_save_and_load_calibration() {
        let colors = Colors::new();
        colors.save_to_file();
        assert!(
            Path::new("calibration.json").exists(),
            "Calibration file should exist after saving"
        );
        if let Some(colors) = Colors::load_from_file() {
            assert_eq!(colors.white(), (255, 255, 255));
            assert_eq!(colors.black(), (0, 0, 0));
            assert_eq!(colors.red(), (255, 0, 0));
            assert_eq!(colors.blue(), (0, 0, 255));
        } else {
            panic!("Should be able to load calibration from file");
        }

        fs::remove_file(Path::new("calibration.json")).unwrap()
    }
}
