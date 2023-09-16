//TODO: Add more unit types to this.
/// Represents a position a motor can travel to.
/// Positions are relative to the last position the motor was zeroed to.
#[derive(Clone, Copy, Debug)]
pub enum Position {
    Degrees(f64),
    Rotations(f64),
    /// Raw encoder ticks.
    Counts(i64),
}

impl Position {
    /// Creates a position from a specified number of degrees.
    pub fn from_degrees(position: f64) -> Self {
        Self::Degrees(position)
    }

    /// Creates a position from a specified number of rotations.
    pub fn from_rotations(position: f64) -> Self {
        Self::Rotations(position)
    }

    /// Creates a position from a specified number of counts (raw encoder tics).
    pub fn from_counts(position: i64) -> Self {
        Self::Counts(position)
    }

    /// Converts a position into degrees.
    pub fn into_degrees(self) -> f64 {
        match self {
            Self::Degrees(num) => num,
            Self::Rotations(num) => num * 360.0,
            Self::Counts(num) => num as f64 * (360.0 / 4096.0),
        }
    }

    /// Converts a position into rotations.
    pub fn into_rotations(self) -> f64 {
        match self {
            Self::Degrees(num) => num / 360.0,
            Self::Rotations(num) => num,
            Self::Counts(num) => num as f64 * 4096.0,
        }
    }

    /// Converts a position into counts (raw encoder ticks).
    pub fn into_counts(self) -> i64 {
        match self {
            Self::Degrees(num) => (num * 4096.0 / 360.0) as i64,
            Self::Rotations(num) => (num * 4096.0) as i64,
            Self::Counts(num) => num,
        }
    }
}
