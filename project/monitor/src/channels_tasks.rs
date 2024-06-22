use common::incident::Incident;

use crate::{camera::Camera, drone::Drone};

// A REFACTORIZAR EN VARIOS ARCHIVOS

/// Represents the action that the UI wants to perform
pub enum UIAction {
    RegistrateDrone(DroneRegistration),
    RegistrateIncident(IncidentRegistration),
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

/// Represents an incident registration
#[derive(Clone)]
pub struct IncidentRegistration {
    pub name: String,
    pub description: String,
    pub x: String,
    pub y: String,
}

/// Represents the action that the monitor wants to perform
pub enum MonitorAction {
    // Connect,
    // Disconnect,
    Drone(Drone),
    Camera(Camera),
    Incident(Incident),
}
