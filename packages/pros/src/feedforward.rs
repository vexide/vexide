/// Feedforward controller for motor control.
///
/// This controller is used to apply feedforward control to achieve desired motor behavior
/// based on velocity and acceleration.
#[derive(Debug, Clone)]
pub struct FeedforwardController {
    /// Feedforward constant for static friction compensation.
    pub ks: f32,
    /// Feedforward constant for velocity compensation.
    pub kv: f32,
    /// Feedforward constant for acceleration compensation.
    pub ka: f32,
    /// Feedforward constant for the target acceleration.
    pub target_acceleration: f32,
    /// Target.
    pub target: f32,
}

impl FeedforwardController {
    /// Creates a new `FeedforwardController` with the given constants and target.
    ///
    /// # Arguments
    ///
    /// * `ks` - Feedforward constant for static friction compensation.
    /// * `kv` - Feedforward constant for velocity compensation.
    /// * `ka` - Feedforward constant for acceleration compensation.
    /// * `target_acceleration` - Feedforward constant for the target acceleration.
    /// * `target` - Target.
    ///
    /// # Returns
    ///
    /// A new `FeedforwardController`.
    pub fn new(ks: f32, kv: f32, ka: f32, target_acceleration: f32, target: f32) -> Self {
        Self {
            ks,
            kv,
            ka,
            target,
        }
    }

    /// Updates the feedforward controller with the current velocity and calculates the control output.
    ///
    /// # Arguments
    ///
    /// * `current_velocity` - The current velocity of the system.
    /// * `target_acceleration` - The target_acceleration of the system.
    /// 
    /// # Returns
    ///
    /// The control output to apply to the motor.
    pub fn update(&mut self, current_velocity: f32, target_acceleration: f32) -> f32 {
        // Calculate the feedforward component based on velocity and acceleration
        let v = self.ks * current_velocity.signum() + self.kv * current_velocity + self.ka * target_acceleration;

        // The output is the feedforward controller (V)
        let output = v;
        
        output
    }
}
