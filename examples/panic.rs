use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    tester();
}

#[inline(never)]
fn tester() {
    panic!("Index out of range!!!!!");
}
