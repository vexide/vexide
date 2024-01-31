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

    pub fn update(&mut self, current_velocity: f32) -> f32 {
        let d_dot = current_velocity;
        let accel = d_dot - self.prev_d_dot;
        self.prev_d_dot = d_dot;

        let error = self.target_rpm - d_dot;
        let proportional = error * self.kp;

        let v = self.ks * d_dot.signum() + self.kv * d_dot + self.ka * accel;

        // The output is the sum of feedback controller (P) and the feedforward controller (V)
        let output = proportional + v;

        output
    }
}
