use crate::hardware::colors::Colors;
use crate::hardware::ui::UI;
use ev3dev_lang_rust::Led;
use ev3dev_lang_rust::sensors::{ColorSensor, GyroSensor, UltrasonicSensor};
use ev3dev_lang_rust::sound::{tone, tone_sequence};
use robolab_bot_base::planet::Direction;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone)]
/// Holds `sensors`
/// - [color_sensor](ColorSensor)
/// - [ultrasonic_sensor](UltrasonicSensor)
/// - [gyro_sensor](GyroSensor)
///
/// and their `calibrations`
/// - [Color calibration](Colors)
/// - `Gyro Calibration`
pub struct Sensors {
    color_sensor: ColorSensor,
    ultrasonic_sensor: UltrasonicSensor,
    gyro_sensor: GyroSensor,
    ref_angle: i32,
    pub colors: Colors,
}

impl Sensors {
    /// Returns a new [Sensors](Sensors) object
    /// to use the sensors [initialization](Sensors::init) is needed
    pub fn new() -> Self {
        Self {
            color_sensor: ColorSensor::find().unwrap(),
            ultrasonic_sensor: UltrasonicSensor::find().unwrap(),
            gyro_sensor: GyroSensor::find().unwrap(),
            ref_angle: 0,
            colors: Colors::new(),
        }
    }

    /// Resets the gyro sensor to zero </br>
    /// Waits for user pressing enter before measuring the gyro drift for 2 seconds </br>
    /// - Gyro not drifting -> `ok` sound and `green` led
    /// - Gyro drifting -> `error` sound and `red` led
    pub fn calibrate_gyro(&self, ui: &UI) -> Result<(), Box<dyn Error>> {
        println!("Calibrating gyro");
        ui.led.set_color(Led::COLOR_YELLOW)?;
        println!("Press OK to calibrate");
        ui.button.process();
        while !ui.button.is_enter() {
            ui.button.process();
        }
        tone(200.0, 250)?;
        sleep(Duration::from_millis(500));
        self.gyro_sensor.set_mode_gyro_rate()?;
        sleep(Duration::from_millis(500));
        self.gyro_sensor.set_mode_gyro_ang()?;
        sleep(Duration::from_millis(500));

        let rotation = self.gyro_sensor.get_angle()?;
        sleep(Duration::from_secs(2));
        let rotation = rotation - self.gyro_sensor.get_angle()?;
        println!("Delta rotation is {rotation}");
        if rotation.abs() < 2 {
            ui.led.set_color(Led::COLOR_GREEN)?;
            tone_sequence(vec![(130.31, 100, 50), (174.61, 100, 50)].as_slice())?;
        } else {
            ui.led.set_color(Led::COLOR_RED)?;
            tone_sequence(vec![(174.61, 100, 50), (130.31, 100, 50)].as_slice())?;
        }

        Ok(())
    }

    /// Calibrates the color sensor </br>
    /// Wants the user to place color sensor above color and press `enter` button </br>
    /// Calibrates white -> black -> blue -> red </br>
    /// Can take multiple samples and average them if needed </br>
    /// `saves` the calibration to a file: [Colors::save_to_file](Colors::save_to_file)
    pub fn calibrate_colors(&mut self, ui: &UI, sample_width: u32) -> Result<(), Box<dyn Error>> {
        println!("Put the color sensor over a white surface");
        self.colors
            .set_white(self.calibrate_color_value(ui, sample_width)?);

        sleep(Duration::from_secs(2));

        println!("Put the color sensor over a black surface");
        self.colors
            .set_black(self.calibrate_color_value(ui, sample_width)?);

        sleep(Duration::from_secs(2));

        println!("Put the color sensor over a blue surface");
        self.colors
            .set_blue(self.calibrate_color_value(ui, sample_width)?);

        sleep(Duration::from_secs(2));

        println!("Put the color sensor over a red surface");
        self.colors
            .set_red(self.calibrate_color_value(ui, sample_width)?);

        println!(
            "calibrated colors: white {:?}, black {:?}, blue {:?}, red {:?}",
            self.colors.white(),
            self.colors.black(),
            self.colors.blue(),
            self.colors.red()
        );

        self.colors.save_to_file();

        Ok(())
    }

    /// Internal helper to calibrate a single color </br>
    /// waits for buttons -> takes sensor value for `sample_width` samples and returns the `average` of them
    fn calibrate_color_value(
        &self,
        ui: &UI,
        sample_width: u32,
    ) -> Result<(i32, i32, i32), Box<dyn Error>> {
        ui.led.set_color(Led::COLOR_RED)?;

        ui.button.process();
        while !ui.button.is_enter() {
            ui.button.process();
        }

        println!("Scanning...");
        ui.led.set_color(Led::COLOR_ORANGE)?;
        let mut color = (0, 0, 0);
        for _ in 0..sample_width {
            let (r, g, b) = self.color_sensor.get_rgb()?;
            color.0 += r;
            color.1 += g;
            color.2 += b;
            sleep(Duration::from_millis(100));
        }

        ui.led.set_color(Led::COLOR_GREEN)?;
        Ok((
            color.0 / sample_width as i32,
            color.1 / sample_width as i32,
            color.2 / sample_width as i32,
        ))
    }

    /// See [Colors::load_from_file](Colors::load_from_file)
    pub fn load_calibrated_colors(&mut self) -> Result<(), Box<dyn Error>> {
        self.colors = Colors::load_from_file().ok_or("Failed to load colors from file")?;
        Ok(())
    }

    /// Returns current ultrasonic distance in `cm` or `Err` if nor readable
    pub fn get_distance(&self) -> Result<i32, Box<dyn Error>> {
        Ok(self.ultrasonic_sensor.get_distance_centimeters()?.floor() as i32)
    }

    /// Returns whether the color sensor is over a path or `Err` if not readable
    pub fn is_path(&self) -> Result<bool, Box<dyn Error>> {
        let (r, g, b) = self.color_sensor.get_rgb()?;
        Ok(self.colors.is_black(r, g, b))
    }

    /// Maps an angle in `deg` to a [Direction](Direction)
    /// - 90° around `0` -> [North](Direction::North)
    /// - 90° around `90` -> [North](Direction::East)
    /// - 90° around `180` -> [North](Direction::South)
    /// - 90° around `270` -> [North](Direction::West)
    pub fn estimate_angle_direction(&self, angle: i32) -> Direction {
        match angle.rem_euclid(360) {
            0..45 => Direction::North,
            45..135 => Direction::East,
            135..225 => Direction::South,
            225..315 => Direction::West,
            315..360 => Direction::North,
            _ => panic!("Direction can't be outside of (0..360) at this point"),
        }
    }

    /// Returns if [color_sensor](ColorSensor) is over red or blue
    /// using [Colors::is_red](Colors::is_red) and [Colors::is_blue](Colors::is_blue)
    pub fn is_node(&self, color: (i32, i32, i32)) -> bool {
        let (r, g, b) = color;

        self.colors.is_red(r, g, b) || self.colors.is_blue(r, g, b)
    }

    /// Returns the angle of the [gyro_sensor](GyroSensor) normalized to [ref_angle](self.set_ref_angle) or `Err` is not readable
    pub fn get_angle(&self) -> Result<i32, Box<dyn Error>> {
        Ok(self.gyro_sensor.get_angle()? - self.ref_angle)
    }

    /// Returns the raw r,g,b color value of the [color_sensor](ColorSensor) or `Err` if not readable
    pub fn get_color(&self) -> Result<(i32, i32, i32), Box<dyn Error>> {
        Ok(self.color_sensor.get_rgb()?)
    }

    /// Returns the [brightness](Colors::get_color_brightness) of the color under the [color_sensor](ColorSensor)
    pub fn get_brightness(&self, color: (i32, i32, i32)) -> i32 {
        self.colors.get_color_brightness(color.0, color.1, color.2)
    }

    /// Sets the direction reference for an angle </br>
    /// internal `ref_angle` gets set to angle at which the [gyro_sensor](GyroSensor) points [North](Direction::North) </br>
    /// ```
    /// let angle = self.get_angle().unwrap();
    /// self.ref_angle = angle - dir_angle;
    /// ```
    pub fn set_ref_angle(&mut self, dir_angle: i32) {
        let angle = self.get_angle().unwrap();
        self.ref_angle = angle - dir_angle;
        println!(
            "angle: {}, dir_angle: {}, ref_angle: {}",
            angle, dir_angle, self.ref_angle
        );
    }

    /// Initializes the sensors to their modes
    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        self.color_sensor.set_mode_rgb_raw()?;
        self.ultrasonic_sensor.set_mode_us_dist_cm()?;
        self.gyro_sensor.set_mode_gyro_ang()?;
        Ok(())
    }
}

impl Default for Sensors {
    /// See [Sensors::new](Sensors::new)
    fn default() -> Self {
        Self::new()
    }
}
