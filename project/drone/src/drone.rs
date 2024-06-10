use crate::{drone_status::DroneStatus, incident::Incident};

const ACTIVE_RANGE: f64 = 20.0;
const MINIMUM_BATTERY_LEVEL: usize = 50;
const MAXIMUM_BATTERY_LEVEL: usize = 100;
const VELOCITY: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct Drone {
    id: u8,
    x_coordinate: f64,
    y_coordinate: f64,
    status: DroneStatus,
    battery: usize,
    x_central: f64,
    y_central: f64,
    x_default: f64,
    y_default: f64,
    actual_incident: Option<(Incident, usize)>,
}

impl Drone {
    pub fn new() -> Self {
        Drone {
            id: 0,
            x_coordinate: 0.0,
            y_coordinate: 0.0,
            status: DroneStatus::Free,
            battery: MAXIMUM_BATTERY_LEVEL,
            x_central: 10.0,
            y_central: 10.0,
            x_default: 0.0,
            y_default: 0.0,
            actual_incident: None,
        }
    }

    pub fn data(&self) -> String {
        format!(
            "{};{};{}",
            self.x_coordinate, self.y_coordinate, self.status
        )
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn is_below_minimun(&self) -> bool {
        self.battery < MINIMUM_BATTERY_LEVEL
    }

    // pub fn update_state(&mut self) {
    //     self.consume_battery();

    //     // if self.is_below_minimun() {
    //     //     self.return_to_central();
    //     // }
    // }

    pub fn consume_battery(&mut self) {
        if self.battery > 0 {
            self.battery -= 10;
        }
    }

    pub fn recharge_battery(&mut self) {
        self.battery = MAXIMUM_BATTERY_LEVEL;
    }

    pub fn set_coordinates(&mut self, x: f64, y: f64) {
        self.x_coordinate = x;
        self.y_coordinate = y;
    }

    pub fn x_coordinate(&self) -> f64 {
        self.x_coordinate
    }

    pub fn y_coordinate(&self) -> f64 {
        self.y_coordinate
    }

    pub fn set_status(&mut self, status: DroneStatus) {
        self.status = status;
    }

    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        euclidean_distance(self.x_coordinate, self.y_coordinate, x, y)
    }

    pub fn x_central_coordinate(&self) -> f64 {
        self.x_central
    }

    pub fn y_central_coordinate(&self) -> f64 {
        self.y_central
    }

    pub fn x_default_coordinate(&self) -> f64 {
        self.x_default
    }

    pub fn y_default_coordinate(&self) -> f64 {
        self.y_default
    }

    pub fn travel_to(&mut self, x: f64, y: f64) {
        let distance = euclidean_distance(self.x_coordinate, self.y_coordinate, x, y);

        if distance > VELOCITY {
            let angle = (y - self.y_coordinate).atan2(x - self.x_coordinate);

            self.x_coordinate += VELOCITY * angle.cos();
            self.y_coordinate += VELOCITY * angle.sin();
        } else {
            self.x_coordinate = x;
            self.y_coordinate = y;
        }
    }

    pub fn is_within_range(&self, x: f64, y: f64) -> bool {
        let distance = euclidean_distance(self.x_default, self.y_default, x, y);

        distance < ACTIVE_RANGE
    }

    pub fn status(&self) -> DroneStatus {
        self.status.clone()
    }

    pub fn set_incident(&mut self, incident: Option<Incident>) {
        match incident {
            Some(incident) => {
                self.actual_incident = Some((incident, 0));
            }
            None => {
                self.actual_incident = None;
            }
        }
    }

    pub fn increment_attending_counter(&mut self) {
        match &mut self.actual_incident {
            Some((_, counter)) => {
                *counter += 1;
            }
            None => {}
        }
    }

    pub fn attending_counter(&self) -> usize {
        match &self.actual_incident {
            Some((_, counter)) => *counter,
            None => 0,
        }
    }

    pub fn incident(&self) -> Option<Incident> {
        match &self.actual_incident {
            Some((incident, _)) => Some(incident.clone()),
            None => None,
        }
    }
}

fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}
