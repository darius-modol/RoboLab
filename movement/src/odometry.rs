use robolab_bot_base::movement::OdometryData;

static SHIFT: u32 = 5;
#[derive(Debug, Clone)]
pub struct Odometry {
    circumference: i32,
    x: i32,
    y: i32,
}

impl Odometry {
    /// Calculates a new position based on
    /// - `d_ticks_(l/r)` delta of [motor_count](ev3dev_lang_rust::motors::LargeMotor::get_position)
    /// - `d_angle` delta of [angle](super::hardware::sensors::Sensors::get_angle)
    /// - `tpr_(l/r)` [ticks per rotation](ev3dev_lang_rust::motors::LargeMotor::get_count_per_rot)
    /// - `angle` current [angle](super::hardware::sensors::Sensors::get_angle)
    pub fn calc_new_pos(
        &mut self,
        d_ticks_l: i32,
        d_ticks_r: i32,
        d_angle: i32,
        tpr_l: i32,
        tpr_r: i32,
        angle: i32,
    ) {
        let tpr_l = if tpr_l == 0 { 360 } else { tpr_l };
        let tpr_r = if tpr_r == 0 { 360 } else { tpr_r };
        let dist_l = ((d_ticks_l << SHIFT) * self.circumference) / tpr_l;
        let dist_r = ((d_ticks_r << SHIFT) * self.circumference) / tpr_r;

        // avg angle
        let arc_avg = (((angle + (angle - d_angle)) / 2) as f32).to_radians();

        // rotational factor
        let arc_fac = if d_angle == 0 {
            1.0
        } else {
            (d_angle as f32 / 2.0).to_radians().sin() / (d_angle as f32 / 2.0).to_radians()
        };

        let lin_dist = (dist_l + dist_r) / 2;

        self.x += (arc_fac * lin_dist as f32 * arc_avg.sin()).round() as i32;
        self.y += (arc_fac * lin_dist as f32 * arc_avg.cos()).round() as i32;
    }

    /// Returns current position </br>
    /// 500 units are 1 planet unit
    /// ```
    /// use movement::odometry::Odometry;
    /// let odometry = Odometry::new(0,0);
    /// let (ox,oy) = odometry.get_pos();
    /// let x =  (ox / 500.0).round() as i32;
    /// let y =  (oy / 500.0).round() as i32;
    /// ```
    pub fn get_pos(&self) -> (f32, f32) {
        (
            self.x as f32 / 2_i32.pow(SHIFT) as f32,
            self.y as f32 / 2_i32.pow(SHIFT) as f32,
        )
    }

    /// Resets the internal position to `(0, 0)`
    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
    }

    /// Deconstructs an [OdometryData](OdometryData) to a tuple </br>
    /// Used for misused `OdometryData` that is internally an (x,y) tuple </br
    /// see [Movement::follow_line](super::movement_impl::Movement::follow_line)
    pub fn odo_to_xy(odometry: OdometryData) -> (i32, i32) {
        let OdometryData {
            motor_position_left,
            motor_position_right,
        } = odometry;
        (motor_position_left, motor_position_right)
    }

    /// Returns a new [Odometry](Odometry) object with `circumference = 176 (mm)` and `x,y`
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            circumference: 176, // 56mm diameter * pi -> 176mm circumference
            x,
            y,
        }
    }
}

impl Default for Odometry {
    /// See [Odometry::new](Odometry::new)
    fn default() -> Self {
        Odometry::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::odometry::Odometry;

    #[test]
    fn test_new_position() {
        let odo = Odometry::new(32, 64); // 32 = 1 << SHIFT, so 1.0 in real coords
        let (x, y) = odo.get_pos();
        assert_eq!(x, 1.0, "Expected x to be 1.0, got {}", x);
        assert_eq!(y, 2.0, "Expected y to be 2.0, got {}", y);
    }

    #[test]
    fn test_drive_straight_north() {
        // sin(0)=0, cos(0)=1 -> only y changes
        let mut odo = Odometry::default();
        odo.calc_new_pos(1024, 1024, 0, 360, 360, 0);
        let (x, y) = odo.get_pos();
        assert!(
            (x - 0.0).abs() < 0.1,
            "Expected x to be approximately 0.0, got {}",
            x
        );
        assert!(
            (y - 500.0).abs() < 20.0,
            "Expected y to be approximately 500, got {}",
            y
        );
    }

    #[test]
    fn test_drive_straight_east() {
        // sin(90)=1, cos(90)=0 -> only x changes
        let mut odo = Odometry::default();
        odo.calc_new_pos(1024, 1024, 0, 360, 360, 90);
        let (x, y) = odo.get_pos();
        assert!(
            (x - 500.0).abs() < 20.0,
            "Expected x to be approximately 500, got {}",
            x
        );
        assert!(
            (y - 0.0).abs() < 0.1,
            "Expected y to be approximately 0.0, got {}",
            y
        );
    }

    #[test]
    fn test_drive_north_east() {
        // in(45)=cos(45)=2*sqrt(2) -> x and y change
        let mut odo = Odometry::default();
        odo.calc_new_pos(1500, 1500, 0, 360, 360, 45);
        let (x, y) = odo.get_pos();
        assert!(
            (x - 500.0).abs() < 20.0,
            "Expected x to be approximately 500, got {}",
            x
        );
        assert!(
            (y - 500.0).abs() < 20.0,
            "Expected y to be approximately 500, got {}",
            y
        );
    }

    #[test]
    fn test_no_movement() {
        let mut odo = Odometry::default();
        odo.calc_new_pos(0, 0, 0, 360, 360, 45);
        let (x, y) = odo.get_pos();
        assert_eq!(x, 0.0, "Expected x to be 0.0, got {}", x);
        assert_eq!(y, 0.0, "Expected y to be 0.0, got {}", y);
    }

    #[test]
    fn test_tpr_zero_fallback() {
        // tpr = 0 should fall back to 360, same result as explicit 360
        let mut odo1 = Odometry::default();
        let mut odo2 = Odometry::default();
        odo1.calc_new_pos(180, 180, 0, 0, 0, 0);
        odo2.calc_new_pos(180, 180, 0, 360, 360, 0);
        assert_eq!(
            odo1.get_pos(),
            odo2.get_pos(),
            "Expected positions to be the same when tpr is 0 or 360"
        );
    }

    #[test]
    fn test_accumulates_position() {
        // Two identical steps north should double the distance
        let mut odo = Odometry::default();
        odo.calc_new_pos(360, 360, 0, 360, 360, 0);
        let (_, y1) = odo.get_pos();
        odo.calc_new_pos(360, 360, 0, 360, 360, 0);
        let (_, y2) = odo.get_pos();
        assert!(
            (y2 - 2.0 * y1).abs() < 0.1,
            "Expected y to approximately double after two steps, got {}",
            y2
        );
    }
}
