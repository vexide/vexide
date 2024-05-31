#![no_main]
#![no_std]

use vexide::{core::time::Instant, prelude::*};

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    use core::hint::black_box;

    println!("{}", f64::sqrt(55.0));
    // unsafe {
    //     let start = Instant::now();
    //     black_box(for _ in 0..10_000_000 {
    //         black_box(sqrt(black_box(35.0)));
    //     });
    //     let elapsed = start.elapsed();
    //     println!("{:?}", elapsed);
    // }
}
