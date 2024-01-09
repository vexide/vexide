//! Physical sensors on the VEX V5.
//!
//! Most sensors can be created with a `new` function that generally takes a port number along with other sensor specific parameters.
//! Multiple sensors can be created on the same port as long as they are all the same sensor type.
//! All sensors are thread safe.
//!
//! In cases where PROS gives the option of a blocking or non-blocking API,
//! the blocking API is used for a synchronous method and the non-blocking API is used to create a future.
//!
//! More specific info for each sensor is availible in their respective modules.
//!
//! Currently supported sensors are:
//! - [`Rotation`](rotation::RotationSensor)
//! - [`Distance`](distance::DistanceSensor)
//! - [`Vision`](vision::VisionSensor)
//! - [`GPS`](gps::GpsSensor)

pub mod distance;
pub mod gps;
pub mod imu;
pub mod optical;
pub mod rotation;
pub mod vision;
