use common::incident::Incident;

#[derive(Debug, PartialEq, Clone)]
pub struct Drone {
    pub id: String,
    pub status: DroneStatus,
    pub battery: usize,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub incident: Option<Incident>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DroneStatus {
    Free,
    AttendingIncident,
    Travelling,
    Recharging,
}

impl DroneStatus {
    pub fn to_str(&self) -> &str {
        match self {
            DroneStatus::Free => "Free",
            DroneStatus::AttendingIncident => "AttendingIncident",
            DroneStatus::Travelling => "Travelling",
            DroneStatus::Recharging => "Charging",
        }
    }
    pub fn from_str(string: &str) -> Self {
        match string {
            "0" => DroneStatus::Free,
            "1" => DroneStatus::AttendingIncident,
            "2" => DroneStatus::Travelling,
            "3" => DroneStatus::Travelling,
            "4" => DroneStatus::Travelling,
            "5" => DroneStatus::Recharging,
            _ => panic!("Invalid drone status"),
        }
    }
}

impl Drone {
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
