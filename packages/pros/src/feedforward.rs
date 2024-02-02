/// Feedforward controller for motor control.
///
/// This controller is used to apply feedforward control to achieve desired motor behavior
/// based on velocity and acceleration.
#[derive(Debug, Clone)]
pub struct FeedforwardMotorController {
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

impl FeedforwardMotorController {
    /// Creates a new [`FeedforwardMotorController`] with the given constants and target.
    ///
    /// # Arguments
    ///
    /// * `ks` - Feedforward constant for static friction compensation.
    /// * `kv` - Feedforward constant for velocity compensation.
    /// * `ka` - Feedforward constant for acceleration compensation.
    /// * `target_acceleration` - Feedforward constant for the target acceleration.
    ///
    /// # Returns
    ///
    /// A new [`FeedforwardMotorController`].
    pub fn new(ks: f32, kv: f32, ka: f32, target_acceleration: f32) -> Self {
        Self {
            ks,
            kv,
            ka,
        }
    }

    /// Updates the feedforward controller to calculate the control output.
    ///
    /// # Arguments
    ///
    /// * `target_acceleration` - The target_acceleration of the system.
    /// * `target` - Target.
    /// 
    /// # Returns
    ///
    /// The control output to apply to the motor.
    pub fn update(&self, target: f32, target_acceleration: f32) -> f32 {
        // Calculate the feedforward component based on velocity and acceleration
        let v = self.ks * target.signum() + self.kv * target + self.ka * target_acceleration;

        // The output is the feedforward controller (V)
        let output = v;
        
        output
    }
}
