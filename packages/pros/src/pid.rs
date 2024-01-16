//! PID controllers.
//!
//! PID controllers are first created with [`PidController::new`]
//! and then can be utilized by calling [`PidController::update`] repeatedly.

/// A proportional–integral–derivative controller.
///
/// This controller is used to smoothly move motors to a certain point,
/// and allows for feedback-based power adjustments. This is desirable
/// over just setting the motor power, as it can be tuned to make the
/// motor stop in exactly the right position without overshooting.
#[derive(Debug, Clone, Copy)]
pub struct PidController {
    /// Proportional constant. This is multiplied by the error to get the
    /// proportional component of the output.
    pub kp: f32,
    /// Integral constant. This accounts for the past values of the error.
    pub ki: f32,
    /// Derivative constant. This allows you to change the motor behavior
    /// based on the rate of change of the error (predicting future values).
    pub kd: f32,

    last_time: i32,
    last_position: f32,
    i: f32,
}

impl PidController {
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            last_time: 0,
            last_position: 0.0,
            i: 0.0,
        }
    }

    pub fn update(&mut self, setpoint: f32, position: f32) -> f32 {
        let time = unsafe { pros_sys::clock() };
        let mut delta_time = (time - self.last_time) as f32 / pros_sys::CLOCKS_PER_SEC as f32;
        if delta_time == 0.0 {
            delta_time += 0.001;
        }
        let error = setpoint - position;

        self.i += error * delta_time;

        let p = self.kp * error;
        let i = self.ki * self.i;

        let mut d = (position - self.last_position) / delta_time;
        if d.is_nan() {
            d = 0.0
        }

        let output = p + i + d;

        self.last_position = position;
        self.last_time = time;

        output
    }
}
