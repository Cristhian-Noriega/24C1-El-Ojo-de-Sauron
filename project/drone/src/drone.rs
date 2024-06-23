use std::collections::VecDeque;

use crate::drone_status::{DroneStatus, TravelLocation};

use common::incident::Incident;

const MINIMUM_BATTERY_LEVEL: usize = 20;
const MAXIMUM_BATTERY_LEVEL: usize = 100;

const BATTERY_UNIT: usize = 1;

/// Represents a drone
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
    incident_queue: VecDeque<Incident>,
    velocity: f64,
    active_range: f64,
}

impl Drone {
    /// Creates a new drone
    pub fn new(
        id: u8,
        x_central: f64,
        y_central: f64,
        x_anchor: f64,
        y_anchor: f64,
        velocity: f64,
        active_range: f64,
    ) -> Self {
        Drone {
            id,
            x_coordinate: x_anchor,
            y_coordinate: y_anchor,
            status: DroneStatus::Free,
            battery: MAXIMUM_BATTERY_LEVEL,
            x_central,
            y_central,
            x_anchor,
            y_anchor,
            current_incident: None,
            incident_queue: VecDeque::new(),
            velocity,
            active_range,
        }
    }

    /// Returns the data of the drone in string format
    pub fn data(&self) -> String {
        format!(
            "{};{};{};{}",
            self.x_coordinate, self.y_coordinate, self.status, self.battery
        )
    }

    /// Returns the id of the drone
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Returns true if the battery is below the minimum level
    pub fn is_below_minimun(&self) -> bool {
        self.battery < MINIMUM_BATTERY_LEVEL
    }

    /// Sets the status of the drone
    pub fn set_status(&mut self, status: DroneStatus) {
        self.status = status;
    }

    /// Calculates the distance to a point
    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        euclidean_distance(self.x_coordinate, self.y_coordinate, x, y)
    }

    /// Returns the x central coordinate of the drone
    pub fn x_central_coordinate(&self) -> f64 {
        self.x_central
    }

    /// Returns the y central coordinate of the drone
    pub fn y_central_coordinate(&self) -> f64 {
        self.y_central
    }

    /// Returns the x anchor coordinate of the drone
    pub fn x_anchor_coordinate(&self) -> f64 {
        self.x_anchor
    }

    /// Returns the y anchor coordinate of the drone
    pub fn y_anchor_coordinate(&self) -> f64 {
        self.y_anchor
    }

    /// Moves the drone to a point
    pub fn travel_to(&mut self, x: f64, y: f64) {
        let distance = self.distance_to(x, y);

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

    /// Discharges the battery of the drone
    pub fn discharge_battery(&mut self) {
        if self.battery > 0 {
            self.battery -= BATTERY_UNIT;
        }
    }

    /// Recharges the battery of the drone
    pub fn recharge_battery(&mut self) {
        if self.battery < MAXIMUM_BATTERY_LEVEL {
            self.battery += BATTERY_UNIT;
        }
    }

    /// Returns true if the battery is fully charged
    pub fn is_fully_charged(&self) -> bool {
        self.battery == MAXIMUM_BATTERY_LEVEL
    }

    /// Returns true if the drone is within range of a point
    pub fn is_within_range(&self, x: f64, y: f64) -> bool {
        let distance = euclidean_distance(self.x_anchor, self.y_anchor, x, y);

        distance < self.active_range
    }

    /// Returns the status of the drone
    pub fn status(&self) -> DroneStatus {
        self.status.clone()
    }

    /// Sets the incident of the drone
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

    /// Increments the attending counter of the drone
    pub fn increment_attending_counter(&mut self) {
        match &mut self.current_incident {
            Some((_, counter)) => {
                *counter += 1;
            }
            None => {}
        }
    }

    /// Returns the attending counter of the drone
    pub fn attending_counter(&self) -> usize {
        match &self.current_incident {
            Some((_, counter)) => *counter,
            None => 0,
        }
    }

    /// Returns the incident of the drone
    pub fn incident(&self) -> Option<Incident> {
        self.current_incident
            .as_ref()
            .map(|(incident, _)| incident.clone())
    }

    /// Returns true if the drone is attending an incident
    pub fn is_travelling_to_incident(&self) -> bool {
        self.status == DroneStatus::Travelling(TravelLocation::Incident)
    }

    pub fn add_incident(&mut self, incident: Incident) {
        self.incident_queue.push_back(incident);
    }

    // pub fn get_next_incident(&mut self) -> Option<Incident> {
    //     self.incident_queue.pop_front()
    // }

    pub fn has_pending_incidents(&self) -> bool {
        !self.incident_queue.is_empty()
    }

    pub fn current_incident(&self) -> Option<Incident> {
        self.incident_queue.front().cloned()
    }

    pub fn remove_current_incident(&mut self) {
        self.incident_queue.pop_front();
    }

    pub fn can_handle_new_incident(&self) -> bool {
        self.status == DroneStatus::Free
            || self.status == DroneStatus::Travelling(TravelLocation::Anchor)
    }
}

/// Calculates the euclidean distance between two points
fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}
