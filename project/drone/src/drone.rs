use crate::drone_status::DroneStatus;

const ACTIVE_RANGE: f64 = 10.0;
const MINIMUM_BATTERY_LEVEL: usize = 50;

#[derive(Debug, Clone)]
pub struct Drone {
    id: u8,
    x_coordinate: f64,
    y_coordinate: f64,
    state: DroneStatus,
    battery: usize,
}

impl Drone {
    pub fn new() -> Self {
        Drone {
            id: 0,
            x_coordinate: 0.0,
            y_coordinate: 0.0,
            state: DroneStatus::Free,
            battery: 100,
        }
    }

    pub fn data(&self) -> String {
        format!("{};{};{}", self.x_coordinate, self.y_coordinate, self.state)
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn is_below_minimun(&self) -> bool {
        self.battery < MINIMUM_BATTERY_LEVEL
    }

    // pub fn is_near(&self, incident: &crate::incident::Incident) -> bool {
    //     let distance = euclidean_distance(
    //         self.x_coordinate,
    //         self.y_coordinate,
    //         incident.x_coordinate,
    //         incident.y_coordinate,
    //     );

    //     distance < ACTIVE_RANGE
    // }
}

fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}
