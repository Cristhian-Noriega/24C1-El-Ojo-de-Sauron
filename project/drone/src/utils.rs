const MINIMUM_BATTERY_LEVEL: usize = 20;
const MAXIMUM_BATTERY_LEVEL: usize = 100;

const BATTERY_DISCHARGE_TRAVELLING: usize = 2;
const BATTERY_DISCHARGE_ATTENDING: usize = 2;
const BATTERY_DISCHARGE_IDLE: usize = 1;
const BATTERY_RECHARGE: usize = 5;

pub struct BatteryConfig {
    pub minimum_level: usize,
    pub maximum_level: usize,
    pub discharge_travelling: usize,
    pub discharge_attending: usize,
    pub discharge_idle: usize,
    pub recharge_rate: usize,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        BatteryConfig {
            minimum_level: MINIMUM_BATTERY_LEVEL,
            maximum_level: MAXIMUM_BATTERY_LEVEL,
            discharge_travelling: BATTERY_DISCHARGE_TRAVELLING,
            discharge_attending: BATTERY_DISCHARGE_ATTENDING,
            discharge_idle: BATTERY_DISCHARGE_IDLE,
            recharge_rate: BATTERY_RECHARGE,
        }
    }
}

/// Represents a position in the 2D space
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    /// Creates a new position
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    /// Calculates the euclidean distance between two points
    pub fn distance_to(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Calculates the angle between two points
    pub fn angle_to(&self, other: &Position) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    /// Moves the position towards another point
    pub fn move_towards(&mut self, other: &Position, distance: f64) {
        let angle = self.angle_to(other);
        self.x += angle.cos() * distance;
        self.y += angle.sin() * distance;
    }
}
