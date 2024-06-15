use crate::{drone_status::DroneStatus, incident::Incident};

const MINIMUM_BATTERY_LEVEL: usize = 95;
const MAXIMUM_BATTERY_LEVEL: usize = 100;
const BATTERY_UNIT: usize = 1;

#[derive(Debug, Clone)]
pub struct Drone {
    id: u8,
    x_coordinate: f64,
    y_coordinate: f64,
    status: DroneStatus,
    battery: usize,
    x_central: f64,
    y_central: f64,
    x_anchor: f64,
    y_anchor: f64,
    current_incident: Option<(Incident, usize)>,
    velocity: f64,
    active_range: f64,
}

impl Drone {
    pub fn new(
        id: u8,
        x_coordinate: f64,
        y_coordinate: f64,
        x_central: f64,
        y_central: f64,
        x_anchor: f64,
        y_anchor: f64,
        velocity: f64,
        active_range: f64,
    ) -> Self {
        Drone {
            id: id,
            x_coordinate: x_coordinate,
            y_coordinate: y_coordinate,
            status: DroneStatus::Free,
            battery: MAXIMUM_BATTERY_LEVEL,
            x_central: x_central,
            y_central: y_central,
            x_anchor: x_anchor,
            y_anchor: y_anchor,
            current_incident: None,
            velocity: velocity,
            active_range: active_range,
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

    pub fn x_anchor_coordinate(&self) -> f64 {
        self.x_anchor
    }

    pub fn y_anchor_coordinate(&self) -> f64 {
        self.y_anchor
    }

    pub fn travel_to(&mut self, x: f64, y: f64) {
        let distance = euclidean_distance(self.x_coordinate, self.y_coordinate, x, y);

        if distance > self.velocity {
            let angle = (y - self.y_coordinate).atan2(x - self.x_coordinate);

            self.x_coordinate += self.velocity * angle.cos();
            self.y_coordinate += self.velocity * angle.sin();
        } else {
            self.x_coordinate = x;
            self.y_coordinate = y;
        }

        self.discharge_battery();
    }

    pub fn discharge_battery(&mut self) {
        if self.battery > 0 {
            println!("Discharging battery: {}", self.battery);
            self.battery -= BATTERY_UNIT;
        }
    }

    pub fn recharge_battery(&mut self) {
        if self.battery < MAXIMUM_BATTERY_LEVEL {
            self.battery += BATTERY_UNIT;
        }
    }

    pub fn is_fully_charged(&self) -> bool {
        self.battery == MAXIMUM_BATTERY_LEVEL
    }

    pub fn is_within_range(&self, x: f64, y: f64) -> bool {
        let distance = euclidean_distance(self.x_anchor, self.y_anchor, x, y);

        distance < self.active_range
    }

    pub fn battery(&self) -> usize {
        self.battery
    }

    pub fn status(&self) -> DroneStatus {
        self.status.clone()
    }

    pub fn set_incident(&mut self, incident: Option<Incident>) {
        match incident {
            Some(incident) => {
                self.current_incident = Some((incident, 0));
            }
            None => {
                self.current_incident = None;
            }
        }
    }

    pub fn increment_attending_counter(&mut self) {
        match &mut self.current_incident {
            Some((_, counter)) => {
                *counter += 1;
            }
            None => {}
        }
    }

    pub fn attending_counter(&self) -> usize {
        match &self.current_incident {
            Some((_, counter)) => *counter,
            None => 0,
        }
    }

    pub fn incident(&self) -> Option<Incident> {
        self.current_incident.as_ref().map(|(incident, _)| incident.clone())
    }
}

fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}
