use vexide::{adi::ADI_UPDATE_INTERVAL, color::Color, prelude::*};

const NUM_PIXELS: usize = 64;

fn wheel(mut wheel_pos: u8) -> Color {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return Color::new(255 - wheel_pos * 3, 0, wheel_pos * 3);
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return Color::new(0, wheel_pos * 3, 255 - wheel_pos * 3);
    }
    wheel_pos -= 170;

    Color::new(wheel_pos * 3, 255 - wheel_pos * 3, 0)
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let mut strip = AdiAddrLed::<NUM_PIXELS>::new(peripherals.adi_a);
    let mut buffer: [Color; NUM_PIXELS] = [Color::BLACK; _];

    loop {
        for j in 0..(256 * 5) {
            for (i, pixel) in buffer.iter_mut().enumerate() {
                *pixel = wheel((((i * 256) as u16 / 10 + j as u16) & 255) as u8);
            }

            strip.set_buffer(buffer.iter().cloned()).unwrap();

            sleep(ADI_UPDATE_INTERVAL).await;
        }
    }
}
