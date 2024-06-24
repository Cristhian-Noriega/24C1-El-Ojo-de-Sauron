use common::{drone_status::DroneStatus, incident::Incident};

/// Represents a drone in the monitor
#[derive(Debug, PartialEq, Clone)]
pub struct Drone {
    pub id: String,
    pub status: DroneStatus,
    pub battery: usize,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub incident: Option<Incident>,
}

impl Drone {
    /// Creates a new drone
    pub fn new(
        id: String,
        status: DroneStatus,
        battery: usize,
        x_coordinate: f64,
        y_coordinate: f64,
    ) -> Self {
        Self {
            id,
            status,
            battery,
            x_coordinate,
            y_coordinate,
            incident: None,
        }
    }
}
