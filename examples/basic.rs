use std::time::Duration;

use vexide::{adi::{ADI_UPDATE_INTERVAL, AdiPort}, prelude::*};

#[vexide::main]
async fn main(peripherals: Peripherals) {
// assume everything is set LOW at this point.
let mut out = AdiDigitalOut::new(peripherals.adi_a);

out.set_high(); // internal_state = HIGH; real_state = internal_state
// -- real_state HIGH, internal_state HIGH, sdk_state LOW --

let port: AdiPort = out.into();

let mut new_out = AdiDigitalOut::new(port); // internal_state = sdk_state
// -- real_state HIGH, internal_state LOW, sdk_state LOW --

new_out.toggle(); // internal_state = !internal_state; real_state = internal_state
// -- real state HIGH, internal state HIGH, sdk state LOW --

sleep(ADI_UPDATE_INTERVAL).await;

new_out.toggle(); // internal_state = !internal_state; real_state = internal_state
// real state LOW, internal state LOW, sdk state HIGH
}
