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
    /// Previous velocity derivative.
    prev_d_dot: f32,
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
            prev_d_dot: 0.0,
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
        // Calculate the derivative of velocity
        let d_dot = current_velocity;
        // Calculate the acceleration
        let accel = d_dot - self.prev_d_dot;
        // Update the previous velocity derivative for the next iteration
        self.prev_d_dot = d_dot;

        // Calculate the error between the target velocity and the current velocity
        let error = self.target_rpm - d_dot;
        // Apply proportional control to correct the error
        let proportional = error * self.kp;

        // Calculate the feedforward component based on velocity and acceleration
        let v = self.ks * d_dot.signum() + self.kv * d_dot + self.ka * accel;

        // The output is the sum of feedback controller (P) and the feedforward controller (V)
        let output = proportional + v;

        output
    }
}
