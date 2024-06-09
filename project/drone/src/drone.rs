use crate::drone_status::DroneStatus;
use std::{sync::{Arc, Mutex}, thread, time::Duration};

const ACTIVE_RANGE: f64 = 10.0;
const MINIMUM_BATTERY_LEVEL: usize = 50;
const MAXIMUM_BATTERY_LEVEL: usize = 100;
const VELOCITY: f64 = 0.1;
const DISCRETE_INTERVAL: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct Drone {
    id: u8,
    x_coordinate: f64,
    y_coordinate: f64,
    state: DroneStatus,
    battery: usize,
    x_central: f64,
    y_central: f64,
}

impl Drone {
    pub fn new() -> Self {
        Drone {
            id: 0,
            x_coordinate: 0.0,
            y_coordinate: 0.0,
            state: DroneStatus::Free,
            battery: MAXIMUM_BATTERY_LEVEL,
            x_central: 10.0,
            y_central: 10.0,
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

    pub fn update_state(&mut self) {
        self.consume_battery();

        // if self.is_below_minimun() {
        //     self.return_to_central();
        // }
    }

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

    pub fn set_state(&mut self, state: DroneStatus) {
        self.state = state;
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
        let distance = euclidean_distance(self.x_coordinate, self.y_coordinate, x, y);

        distance < ACTIVE_RANGE
    }
}

fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}
