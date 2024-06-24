use common::incident::Incident;

use crate::{camera::Camera, drone::Drone};

/// Represents the action that the UI wants to perform
pub enum UIAction {
    RegistrateDrone(DroneRegistration),
    RegistrateIncident(IncidentRegistration),
    EditIncident(IncidentEdit),
    ResolveIncident(Incident),
}

/// Represents a drone registration
#[derive(Clone)]
pub struct DroneRegistration {
    pub id: String,
    pub username: String,
    pub password: String,
}

impl DroneRegistration {
    pub fn build_drone_message(&self) -> String {
        format!("{};{};{}", self.id, self.username, self.password)
    }
}

/// Represents the form to edit a drone
#[derive(Clone)]
pub struct IncidentRegistration {
    pub name: String,
    pub description: String,
    pub x: String,
    pub y: String,
}

/// Represents the form to edit an incident
#[derive(Clone)]
pub struct IncidentEdit {
    pub uuid: String,
    pub name: String,
    pub description: String,
}

/// Represents the action that the monitor wants to perform
pub enum MonitorAction {
    Drone(Drone),
    Camera(Camera),
    Incident(Incident),
}
