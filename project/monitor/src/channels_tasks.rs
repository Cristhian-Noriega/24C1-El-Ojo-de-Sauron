use common::incident::Incident;

use crate::{camera::Camera, drone::Drone};

// A REFACTORIZAR EN VARIOS ARCHIVOS
pub enum UIAction {
    RegistrateDrone(DroneRegistration),
    RegistrateIncident(IncidentRegistration),
    ResolveIncident(Incident),
}

#[derive(Clone)]
pub struct DroneRegistration {
    pub id: String,
    pub password: String,
}

impl DroneRegistration {
    pub fn build_drone_message(&self) -> String {
        format!(
            "{};{}",
            self.id, self.password
        )
    }
}

#[derive(Clone)]
pub struct IncidentRegistration {
    pub name: String,
    pub description: String,
    pub x: String,
    pub y: String,
}

pub enum MonitorAction {
    // Connect,
    // Disconnect,
    DroneData(Drone),
    CameraData(Camera),
    IncidentData(Incident),
}
