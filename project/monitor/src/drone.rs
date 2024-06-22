use common::incident::Incident;

/// Represents a drone in the monitor
#[derive(Debug, PartialEq, Clone)]
pub struct Drone {
    pub id: String,
    pub state: String,
    pub battery: usize,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub incident: Option<Incident>,
}

impl Drone {
    /// Creates a new drone
    pub fn new(
        id: String,
        state: String, //pasarlo quizas a enum
        battery: usize,
        x_coordinate: f64,
        y_coordinate: f64,
    ) -> Self {
        Self {
            id,
            state,
            battery,
            x_coordinate,
            y_coordinate,
            incident: None,
        }
    }
}
