pub struct PidController {
    pub kp: f32,
    pub ki: f32,
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
        let delta_time = (time - self.last_time) as f32 / pros_sys::CLOCKS_PER_SEC as f32;
        let error = setpoint - position;

        self.i += error * delta_time;

        let p = self.kp * error;
        let i = self.ki * self.i;

        let mut d = (position - self.last_position) / delta_time;

        if d == f32::NAN {
            d = 0.0;
        }
        
        let output = p + i + d;

        self.last_position = position;
        self.last_time = time;

        output
    }
}
