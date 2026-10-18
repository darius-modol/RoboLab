use std::collections::VecDeque;

const SCALE_SHIFT: u32 = 10;

#[derive(Debug, Clone)]
/// PID Controller
pub struct Pid {
    history: VecDeque<(i32, u32)>,
    integral: i32,
    anchor: i32,
    d_window: usize,
    kp: i32,
    ki: i32,
    kd: i32,
}

impl Pid {
    /// Creates a new [PID Controller](Pid) with default values and given `kp, ki, kde`
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            history: VecDeque::new(),
            integral: 0,
            anchor: 50,
            d_window: 10,
            kp: (kp * 2_i32.pow(SCALE_SHIFT) as f32) as i32,
            ki: (ki * 2_i32.pow(SCALE_SHIFT) as f32) as i32,
            kd: (kd * 2_i32.pow(SCALE_SHIFT) as f32) as i32,
        }
    }

    /// Calculates ratio between motors based on the [color_sensor](ev3dev_lang_rust::sensors::ColorSensor)s
    /// [brightness](super::hardware::sensors::Sensors::get_brightness) and the current `time`
    pub fn calculate_ratio(&mut self, brightness: i32, tick_ms: u32) -> (i32, i32) {
        let error = brightness - self.anchor;

        let p = self.calc_p(error);
        let i = self.calc_i(error, tick_ms);
        let d = self.calc_d(error, tick_ms);

        let ratio = (self.kp * p + (self.ki * i) + (self.kd * d)) >> SCALE_SHIFT;

        if self.history.len() > self.d_window {
            self.history.pop_back();
        }
        self.history.push_front((error, tick_ms));

        (ratio.clamp(-50, 50), i)
    }

    /// Resets the internal state of the [PID Controller](Pid)
    pub fn reset(&mut self) {
        self.history.clear();
        self.integral = 0;
    }

    /// Calculates the proportional part based on the current `error` xD
    fn calc_p(&self, error: i32) -> i32 {
        error
    }

    /// Calculates the integral based on current `error` and `time`
    /// If history isn't available, the integral is `zero`
    fn calc_i(&mut self, error: i32, tick_ms: u32) -> i32 {
        // If history exists we add the current error to the integral
        if let Some((_, t)) = self.history.front() {
            let dt = tick_ms.wrapping_sub(*t) as i32;
            self.integral += error * dt;
            self.integral = self.integral.clamp(-1000, 1000)
        }
        self.integral
    }

    /// Calculates the derivative based on current `error` and `time` </br>
    /// Use `d_window` amount of entries if possible and `all` available if less
    /// If history isn't available, the derivative is `zero`
    fn calc_d(&self, error: i32, tick_ms: u32) -> i32 {
        // Use `d_window` amount of entries if possible and `all` available if less
        let d_window = self.d_window.min(self.history.len());

        if d_window == 0 {
            // No history -> no derivative
            return 0;
        }

        if let Some((old_err, old_tick)) = self.history.get(d_window - 1) {
            let delta_e = error - old_err;
            let delta_t = tick_ms.wrapping_sub(*old_tick) as i32;
            if delta_t == 0 {
                0
            } else {
                // (delta_e * scale) / millis
                ((delta_e << (SCALE_SHIFT - 5)) / delta_t).clamp(-100, 100)
            }
        } else {
            // Shouldn't happen but if we ignore the derivative
            0
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    // Test the Proportional term

    /// Simplest test which checks that we get a zero ratio when in the middle of the line
    #[test]
    fn test_pid_at_anchor_returns_zero() {
        let mut pid = Pid::new(1.0, 1.0, 1.0);
        let ratio = pid.calculate_ratio(50, 0).0;
        assert!(
            ratio.abs() < 1,
            "Invalid PID calculation  - expected 0, got {ratio}"
        );
    }

    /// check we are returning to line if too much on white
    #[test]
    fn test_proportional_positive_error() {
        let mut pid = Pid::new(1.0, 0.0, 0.0);
        let ratio = pid.calculate_ratio(80, 0).0;
        assert!(
            ratio > 0,
            "Brightness above anchor should give positive ratio, got {ratio}"
        );
    }

    /// check we are leaving line if too much on black
    #[test]
    fn test_proportional_negative_error() {
        let mut pid = Pid::new(1.0, 0.0, 0.0);
        let ratio = pid.calculate_ratio(20, 0).0;
        assert!(
            ratio < 0,
            "Brightness below anchor should give negative ratio, got {ratio}"
        );
    }

    /// ratio should be symmetric to the anchor
    #[test]
    fn test_proportional_symmetry() {
        let mut pid = Pid::new(1.0, 0.0, 0.0);
        let r1 = pid.calculate_ratio(70, 0).0;
        let r2 = pid.calculate_ratio(30, 0).0;
        assert!(
            (r1 + r2).abs() < 1,
            "Symmetric inputs should give opposite outputs, got {r1} and {r2}"
        );
    }

    // Test the Derivative term

    /// When input is not changing, d should be zero
    #[test]
    fn test_derivative_zero_on_constant_input() {
        let mut pid = Pid::new(0.0, 0.0, 1.0);
        pid.calculate_ratio(70, 0);
        pid.calculate_ratio(70, 100);
        let ratio = pid.calculate_ratio(70, 200).0;
        assert!(
            ratio.abs() < 1,
            "D-term should be 0 for constant input, got {ratio}"
        );
    }

    /// When input is changing, d should not be zero
    #[test]
    fn test_derivative_nonzero_on_changing_input() {
        let mut pid = Pid::new(0.0, 0.0, 1.0);
        pid.calculate_ratio(30, 0);
        pid.calculate_ratio(50, 100);
        let ratio = pid.calculate_ratio(70, 200).0;
        assert!(
            ratio.abs() > 1,
            "D-term should be !0 for changing input, got {ratio}"
        );
    }

    /// When error is increasing from 0, d should be positive
    #[test]
    fn test_derivative_sign_for_increasing_error() {
        let mut pid = Pid::new(0.0, 0.0, 1.0);
        pid.calculate_ratio(50, 0);
        let ratio = pid.calculate_ratio(70, 100).0;
        assert!(
            ratio > 0,
            "D-term should be positive for increasing error, got {ratio}"
        );
    }

    /// When error is decreasing from 0, d should be negative
    #[test]
    fn test_derivative_sign_for_decreasing_error() {
        let mut pid = Pid::new(0.0, 0.0, 1.0);
        pid.calculate_ratio(70, 0);
        let ratio = pid.calculate_ratio(50, 100).0;
        assert!(
            ratio < 0,
            "D-term should be negative for decreasing error, got {ratio}"
        );
    }

    // Test integral part

    /// Integral should accumulate over time and increase ratio
    #[test]
    fn test_integral_accumulates_over_time_positive() {
        let mut pid = Pid::new(0.0, 1.0, 0.0);
        let r1 = pid.calculate_ratio(80, 0).0;
        let r2 = pid.calculate_ratio(80, 100).0;
        let r3 = pid.calculate_ratio(80, 200).0;
        assert!(
            r3.abs() >= r2.abs() && r2.abs() >= r1.abs(),
            "I-term should grow over time with constant error, got {r1}, {r2}, {r3}"
        );
    }

    /// Integral sign should be positive for sustained positive error
    #[test]
    fn test_integral_positive_for_positive_error() {
        let mut pid = Pid::new(0.0, 1.0, 0.0);
        pid.calculate_ratio(80, 0);
        pid.calculate_ratio(80, 100);
        let ratio = pid.calculate_ratio(80, 200).0;
        assert!(
            ratio > 0,
            "I-term should be positive for sustained positive error, got {ratio}"
        );
    }

    /// Integral windup should be clamped — ratio should not grow unboundedly
    #[test]
    fn test_integral_windup_clamped() {
        let mut pid = Pid::new(0.0, 1.0, 0.0);
        // Accumulate over a very long time
        for i in 0..100 {
            pid.calculate_ratio(100, i * 1000);
        }
        let ratio = pid.calculate_ratio(100, 100 * 1000).0;
        assert_eq!(
            ratio, 50,
            "I-term windup should be clamped to max output, got {ratio}"
        );
    }

    // Test clamping

    /// Output should be clamped to 50 when brightness ~= 100
    #[test]
    fn test_output_clamped_high() {
        let mut pid = Pid::new(10.0, 0.0, 0.0);
        let ratio = pid.calculate_ratio(100, 0).0;
        assert_eq!(ratio, 50, "Output should be clamped to 50, got {ratio}");
    }

    /// Output should be clamped to -50 when brightness ~< 0
    #[test]
    fn test_output_clamped_low() {
        let mut pid = Pid::new(10.0, 0.0, 0.0);
        let ratio = pid.calculate_ratio(0, 0).0;
        assert_eq!(ratio, -50, "Output should be clamped to -50, got {ratio}");
    }

    // Test Reset

    /// After reset, the PID should return to initial state and give ca. 0 ratio for anchor input
    #[test]
    fn test_reset() {
        let mut pid = Pid::new(0.0, 1.0, 0.0);
        pid.calculate_ratio(80, 0);
        pid.calculate_ratio(80, 100);
        pid.reset();
        let ratio = pid.calculate_ratio(50, 200).0;
        assert!(
            ratio.abs() < 1,
            "After reset, anchor input should give ~0 ratio, got {ratio}"
        );
    }
}
