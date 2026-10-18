use ev3dev_lang_rust::motors::{LargeMotor, MediumMotor, MotorPort};
use std::error::Error;

#[derive(Debug, Clone)]
/// Holds and controls two large motors
pub struct Motors {
    motor_l: LargeMotor,
    motor_r: LargeMotor,
    motor_fun: Option<MediumMotor>,
}

impl Motors {
    pub fn new() -> Self {
        Self {
            motor_l: LargeMotor::get(MotorPort::OutD).unwrap(),
            motor_r: LargeMotor::get(MotorPort::OutA).unwrap(),
            motor_fun: MediumMotor::get(MotorPort::OutB).ok(),
        }
    }

    /// Sets the mode both motors to [run_direct](LargeMotor::run_direct)
    pub fn start_motors(&self) -> Result<(), Box<dyn Error>> {
        self.motor_l.run_direct()?;
        self.motor_r.run_direct()?;
        if let Some(medium) = &self.motor_fun {
            let _ = medium.run_direct();
            let _ = medium.set_duty_cycle_sp(100);
        }
        Ok(())
    }

    /// Set [speed](LargeMotor::set_duty_cycle_sp) of the motors to `speed_l` and `speed_r` clamped to `-100..=100`
    pub fn set_speed(&self, speed_l: i32, speed_r: i32) -> Result<(), Box<dyn Error>> {
        self.motor_l.set_duty_cycle_sp(speed_l.clamp(-100, 100))?;
        self.motor_r.set_duty_cycle_sp(speed_r.clamp(-100, 100))?;
        Ok(())
    }

    /// Stops the motors
    pub fn stop_motors(&self) -> Result<(), Box<dyn Error>> {
        self.motor_l
            .set_stop_action(LargeMotor::STOP_ACTION_BRAKE)?;
        self.motor_r
            .set_stop_action(LargeMotor::STOP_ACTION_BRAKE)?;
        self.motor_l.stop()?;
        self.motor_r.stop()?;
        if let Some(medium) = &self.motor_fun {
            let _ = medium.stop();
        }
        Ok(())
    }

    /// Returns the [ticks per rotation](LargeMotor::get_count_per_rot) of the motors
    pub fn get_tpr(&self) -> Result<(i32, i32), Box<dyn Error>> {
        Ok((
            self.motor_l.get_count_per_rot()?,
            self.motor_r.get_count_per_rot()?,
        ))
    }

    /// Returns the [position](LargeMotor::get_position) of the motors
    pub fn get_positions(&self) -> Result<(i32, i32), Box<dyn Error>> {
        Ok((self.motor_l.get_position()?, self.motor_r.get_position()?))
    }
}

impl Default for Motors {
    /// See [Motors::new](Motors::new)
    fn default() -> Self {
        Self::new()
    }
}
