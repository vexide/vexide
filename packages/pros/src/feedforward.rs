
use std::time::{Instant, Duration};

/// Feedforward controller for motor control.
///
/// This controller is used to apply feedforward control to achieve desired motor behavior
/// based on velocity and acceleration.
#[derive(Debug, Clone, Copy)]
pub struct FeedforwardController {
    /// Feedforward constant for static friction compensation.
    pub ks: f32,
    /// Feedforward constant for velocity compensation.
    pub kv: f32,
    /// Feedforward constant for acceleration compensation.
    pub ka: f32,
    /// Proportional constant for error correction.
    pub kp: f32,
    /// Target velocity in RPM.
    pub target_rpm: f32,
    /// Previous velocity measurement.
    prev_velocity: f32,
    /// Previous time stamp.
    last_time: Instant,
}

impl FeedforwardController {
    /// Creates a new `FeedforwardController` with the given constants and target RPM.
    ///
    /// # Arguments
    ///
    /// * `ks` - Feedforward constant for static friction compensation.
    /// * `kv` - Feedforward constant for velocity compensation.
    /// * `ka` - Feedforward constant for acceleration compensation.
    /// * `kp` - Proportional constant for error correction.
    /// * `target_rpm` - Target velocity in RPM.
    ///
    /// # Returns
    ///
    /// A new `FeedforwardController`.
    pub fn new(ks: f32, kv: f32, ka: f32, kp: f32, target_rpm: f32) -> Self {
        Self {
            ks,
            kv,
            ka,
            kp,
            target_rpm,
            prev_velocity: 0.0,
            last_time: Instant::now(),
        }
    }

    /// Updates the feedforward controller with the current velocity and calculates the control output.
    ///
    /// # Arguments
    ///
    /// * `current_velocity` - The current velocity of the system in RPM.
    ///
    /// # Returns
    ///
    /// The control output voltage to apply to the motor.
    pub fn update(&mut self, current_velocity: f32) -> f32 {
        // Calculate the time elapsed since the last update
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_time).as_secs_f32();
        let delta_time = if delta_time == 0.0 { 0.001 } else { delta_time };
        self.last_time = now;

        // Calculate the acceleration
        let accel = (current_velocity - self.prev_velocity) / delta_time;
        self.prev_velocity = current_velocity;

        // Calculate the error between the target velocity and the current velocity
        let error = self.target_rpm - current_velocity;

        // Apply proportional control to correct the error
        let proportional = error * self.kp;

        // Calculate the feedforward component based on velocity and acceleration
        let v = self.ks * current_velocity.signum() + self.kv * current_velocity + self.ka * accel;

        // The output is the sum of feedback controller (P) and the feedforward controller (V)
        let output = proportional + v;
        
        output
    }
}
