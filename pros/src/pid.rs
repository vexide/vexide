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

    pub fn update(&mut self, setpoint: f32, postion: f32) -> f32 {
        let time = unsafe { pros_sys::clock() };
        let delta_time = (time - self.last_time) as f32 / pros_sys::CLOCKS_PER_SEC as f32;
        let error = setpoint - postion;

        self.i += error * delta_time;

        let output = (self.kp * error) + (self.ki * self.i) + (self.kd * ((postion - self.last_position) / delta_time));

        self.last_position = postion;
        self.last_time = time;

        output
    }
}