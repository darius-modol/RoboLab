use ev3dev_lang_rust::sound::play;
use ev3dev_lang_rust::Led;
use robolab_bot_base::movement::OdometryData;
use robolab_bot_base::planet::Direction;
use robolab_bot_base::{movement::BotMovement, RlMovement};
use std::cmp::PartialEq;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::hardware::motors::Motors;
use crate::hardware::sensors::Sensors;
use crate::hardware::ui::UI;
use crate::odometry::Odometry;
use crate::pid_controller::Pid;

/// Contains the hardware (motors, sensors) of the robot and
/// also provides calculation methods for "Odometry"
#[derive(Debug, RlMovement)]
pub(super) struct Movement {
    pub motors: Motors,
    pub sensors: Sensors,
    pub ui: Rc<UI>,
    pid: Pid,
    odometry: Odometry,
    speed: i32,
    state: State,
}

#[derive(Debug, Clone, PartialEq)]
enum State {
    Uninitialized,
    PartiallyInitialized,
    FullyInitialized,
}

impl BotMovement for Movement {
    // WARN: DO NOT EDIT FUNCTION SIGNATURE.
    // You can edit the body of this function however you want but the function signature MUST NOT be changed.
    /// Scans a node for adjacent paths in all four [directions](Direction)
    fn scan_nodes(&mut self) -> [Option<Direction>; 4] {
        if let Ok(res) = self.scan_node_impl() {
            res
        } else {
            eprintln!("Error in node exploration");
            Default::default()
        }
    }

    // WARN: DO NOT EDIT FUNCTION SIGNATURE.
    // You can edit the body of this function however you want but the function signature MUST NOT be changed.
    /// Follows a line until a [node](robolab_bot_base::planet::Node) is reached
    /// If it encounters an obstacle it rotates 180° and returns to the start node
    /// - returns `vec!((delta_x, delta_y), (end_angle, 0))` if path was free
    /// - returns `None` if the path was blocked
    ///
    /// # SPECIAL
    /// - The first call also initializes motors and sensors
    /// - The second call doesn't move but sets the gyro-sensor's
    ///   [ref_angle](Sensors::set_ref_angle) based on the given [direction](Direction)
    /// - All calls after the second turn to [direction](Direction) and follow a line. Those return valid [odometry-data](OdometryData)
    ///
    /// **Thanks for your `excellent` template that basically forced be to do such shit** </br>
    /// **Also "thanks" for [OdometryData](OdometryData) to NOT derive Debug or Clone**
    fn follow_line(&mut self, start_direction: Direction) -> Option<Vec<OdometryData>> {
        // Start with clean PID and odometry -> odometry is delta
        self.pid.reset();
        self.odometry.reset();

        match self.state {
            // Setup sensors and motors and follow line to fist node -> set state to PartiallyInitialized
            State::Uninitialized => self.initialize_line_following(),
            // Set gyro's ref_angle but do not move -> set state to FullyInitialized
            State::PartiallyInitialized => {
                self.sensors.set_ref_angle(start_direction as i32);
                println!("Initialized base rotation");
                self.state = State::FullyInitialized;
                return None;
            }
            // Rotate to given direction and follow line
            State::FullyInitialized => {
                self.rotate_to_direction(start_direction).ok()?;
            }
        }

        let start_time = Instant::now();
        self.follow_line_impl(start_time)
    }
}

impl Movement {
    /// Create a new [Movement](Movement) object with preconfigures [PID](Pid) constants
    pub fn new(speed: i32, ui: Rc<UI>) -> Self {
        let kp = 0.7;
        let ki = 0.001; //0.005;
        let kd = 0.7; //0.6;

        Self {
            motors: Motors::default(),
            sensors: Sensors::new(),
            ui,
            pid: Pid::new(kp, ki, kd),
            odometry: Odometry::new(0, 0),
            speed,
            state: State::Uninitialized,
        }
    }

    /// Follows a line until a [node](robolab_bot_base::planet::Node) is reached
    /// If it encounters an obstacle it rotates 180° and returns to the start node
    /// - returns `vec!((delta_x, delta_y), (end_angle, 0))` if path was free
    /// - returns `None` if the path was blocked
    pub fn follow_line_impl(&mut self, start_time: Instant) -> Option<Vec<OdometryData>> {
        self.motors.start_motors().ok()?;
        // state for evaluating blocked paths
        let mut blocked = false;

        // motor position history for odometry calculation
        let mut motor_positions: Vec<(i32, i32, i32)> = Vec::new();

        // loop until node or backspace
        while !self.ui.button.is_backspace() {
            self.ui.button.process();

            // update odometry calculations
            self.calculate_odometry(&mut motor_positions).ok()?;

            let color = self.sensors.get_color().ok()?;
            let brightness = self.sensors.get_brightness(color);
            let tick_ms = start_time.elapsed().as_millis() as u32;

            // calculate desired delta / ratio between motors to stay on track
            let (ratio, _) = self.pid.calculate_ratio(brightness, tick_ms);

            // If the path is blocked we set it blocked and return to the start
            if self.sensors.get_distance().ok()? < 20 {
                println!("Blocked Path!");
                let _ = play("./sounds/path-blocked/windows.wav");
                blocked = true;
                sleep(Duration::from_millis(500));
                let current_rot = self.sensors.get_angle().ok()?;
                self.motors.set_speed(40, -40).ok();
                while self.sensors.get_angle().ok()? <= current_rot + 180 {
                    sleep(Duration::from_millis(20));
                }
                continue;
            }

            // If we reach a node the line following is finished
            if self.sensors.is_node(color) {
                println!("Reached node!");

                // Cleanly drive to the middle of the node
                self.motors.set_speed(self.speed, self.speed).unwrap();
                sleep(Duration::from_millis(400));

                self.motors.set_speed(0, 0).unwrap();
                self.motors.stop_motors().unwrap();

                break;
            }

            //let speed = (self.speed / (integral as f32 * 0.005).round() as i32).clamp(20, 50);

            // Set motors to calculated speed
            self.motors
                .set_speed(self.speed - ratio, self.speed + ratio)
                .ok();
        }

        // Return OdometryData
        if blocked {
            None
        } else {
            let (x, y) = self.odometry.get_pos();
            let dir = self
                .sensors
                .estimate_angle_direction(self.sensors.get_angle().ok()? + 180);
            // Hacky way to encode odometry result into Vec<OdometryData>
            Some(vec![
                OdometryData {
                    motor_position_left: (x / 500.0).round() as i32,
                    motor_position_right: (y / 500.0).round() as i32,
                },
                OdometryData {
                    motor_position_left: dir as i32,
                    motor_position_right: 0,
                },
            ])
        }
    }

    /// Scans a node for adjacent paths in all four [directions](Direction)
    pub fn scan_node_impl(&mut self) -> Result<[Option<Direction>; 4], Box<dyn Error>> {
        self.motors.start_motors()?;
        let turn_speed = 40;
        // Force this shit to turn
        sleep(Duration::from_millis(50));
        self.motors.set_speed(turn_speed, -turn_speed)?;
        sleep(Duration::from_millis(50));
        self.motors.set_speed(turn_speed, -turn_speed)?;
        sleep(Duration::from_millis(50));
        self.motors.set_speed(turn_speed, -turn_speed)?;
        sleep(Duration::from_millis(50));

        let mut directions = [None; 4];

        // rotate - check if angle is a direction and path exists -> set path available
        let initial_angle = self.sensors.get_angle()?;
        while let Ok(angle) = self.sensors.get_angle()
            && angle < initial_angle + 360
        {
            if self.sensors.is_path()? {
                let direction = self.sensors.estimate_angle_direction(angle);
                match direction {
                    Direction::North => directions[0] = Some(direction),
                    Direction::East => directions[1] = Some(direction),
                    Direction::South => directions[2] = Some(direction),
                    Direction::West => directions[3] = Some(direction),
                }
            }
        }

        // Force this shit to stop
        self.motors.stop_motors()?;
        sleep(Duration::from_millis(250));
        self.motors.stop_motors()?;
        sleep(Duration::from_millis(250));
        self.motors.stop_motors()?;
        sleep(Duration::from_millis(250));

        Ok(directions)
    }

    /// Rotates to a give [Direction](Direction)
    fn rotate_to_direction(&self, direction: Direction) -> Result<(), Box<dyn Error>> {
        self.motors.start_motors()?;

        let direction = direction as i32;
        println!(
            "Starting rotation: {}, {}",
            self.sensors.get_angle()?,
            direction
        );

        // Overshoot a little to catch the line after rotating
        while !(5..15).contains(&((self.sensors.get_angle()? - direction).rem_euclid(360))) {
            self.motors.set_speed(40, -40)?;
            sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    /// Update the internal data in `self.odometry`
    fn calculate_odometry(
        &mut self,
        motor_positions: &mut Vec<(i32, i32, i32)>,
    ) -> Result<(), Box<dyn Error>> {
        let (m_l, m_r) = self.motors.get_positions()?;
        let angle = self.sensors.get_angle()?.rem_euclid(360);

        // Calculate deltas
        let (d_tick_l, d_tick_r, d_angle) = match motor_positions.last() {
            Some((old_l, old_r, old_ang)) => (m_l - old_l, m_r - old_r, angle - old_ang),
            None => (0, 0, 0),
        };

        let (tpr_l, tpr_r) = self.motors.get_tpr()?;

        // Calculate the new position
        self.odometry
            .calc_new_pos(d_tick_l, d_tick_r, d_angle, tpr_l, tpr_r, angle);

        // Push current motor pos for use in next loop iteration
        motor_positions.push((m_l, m_r, angle));
        Ok(())
    }

    /// initializes movement struct - see [setup_movement](Movement::setup_movement) </br>
    /// starts the motors
    fn initialize_line_following(&mut self) {
        if let Some(e) = self.setup_movement().err() {
            println!("Failed to setup movement {}", e)
        }

        self.motors.start_motors().ok();

        self.state = State::PartiallyInitialized
    }

    /// Setups sensors and motors </br>
    /// - [Calibrates](Sensors::calibrate_colors) color_sensor or [loads](Sensors::load_calibrated_colors) it's config from file
    /// - [Calibrates](Sensors::calibrate_gyro) gyro_sensor
    fn setup_movement(&mut self) -> Result<(), Box<dyn Error>> {
        self.sensors.init()?;

        let path = PathBuf::from(Path::new("./calibration.json"));
        // check existence of config file - if it exists user can choose to load or recalibrate
        if path.exists() {
            self.ui.led.set_color(Led::COLOR_AMBER)?;
            println!("Config exists - press OK to load / DOWN to recalibrate");

            self.ui.button.process();
            while !self.ui.button.is_enter() && !self.ui.button.is_down() {
                self.ui.button.process()
            }

            if self.ui.button.is_enter() {
                self.sensors.load_calibrated_colors()?;
                self.ui.led.set_color(Led::COLOR_GREEN)?;
            } else if self.ui.button.is_down() {
                self.sensors.calibrate_colors(&self.ui, 1)?;
            }
        } else {
            println!("No config existent - calibrating");
            self.sensors.calibrate_colors(&self.ui, 1)?;
        }

        self.motors.set_speed(self.speed, self.speed)?;
        println!("Place the robot on the track");
        self.sensors.calibrate_gyro(&self.ui)?;
        Ok(())
    }
}
