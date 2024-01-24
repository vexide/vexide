#![no_std]
#![no_main]

extern crate alloc;

use core::time::Duration;

use alloc::vec;

use pros::{devices::smart::vision::Rgb, prelude::*};

pub struct Robot {
    led_strip: AdiAddrLed,
}

fn wheel(mut wheel_pos: u8) -> Rgb {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return Rgb::new(255 - wheel_pos * 3, 0, wheel_pos * 3);
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return Rgb::new(0, wheel_pos * 3, 255 - wheel_pos * 3);
    }
    wheel_pos -= 170;

    Rgb::new(wheel_pos * 3, 255 - wheel_pos * 3, 0)
}


impl Robot {
    pub fn new(peripherals: Peripherals) -> Self {
        Self {
            led_strip: AdiAddrLed::new(peripherals.adi_d, vec![Rgb::new(0, 0, 0); 10])
                .unwrap_or_else(|err| {
                    panic!("Failed to construct LED strip! Error: {err:?}");
                }),
        }
    }
}

impl SyncRobot for Robot {
    fn opcontrol(&mut self) -> pros::Result {
        println!("==== ADDRLED TEST ====");
        println!("This will run a few basic test procedures on an AddrLed strip.");
        println!("There will be a two second sleep between each procedure.\n");

        println!("Test 1: Setting all LEDs to rgb(255, 0, 0).");
        self.led_strip.set_all(Rgb::new(255, 0, 0)).unwrap();

        println!("Test 2: Clearing all LEDs");
        self.led_strip.clear_all().unwrap();

        println!("Test 3: Setting first LED to rgb(255, 0, 0).");
        self.led_strip.set_pixel(0, Rgb::new(255, 0, 0)).unwrap();

        println!("Test 4: Clearing first pixel");
        self.led_strip.clear_pixel(0).unwrap();

        println!("Test 5: Rainbow Buffer");
        let mut data = [0u32; 10];

        loop {
            for j in 0..(256 * 5) {
                for i in 0..10 {
                    data[i] = wheel((((i * 256) as u16 / 10 as u16 + j as u16) & 255) as u8).into();
                }

				self.led_strip.set_buffer(data.iter().cloned()).unwrap();

                pros::task::delay(Duration::from_millis(10));
            }
        }

        Ok(())
    }
}

sync_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
