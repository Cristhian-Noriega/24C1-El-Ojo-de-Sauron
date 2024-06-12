use crate::{camera::Camera, drone::Drone, incident::Incident};

// A REFACTORIZAR EN VARIOS ARCHIVOS
pub enum UIAction {
    // Connect,
    // Disconnect,
    RegistrateDrone(DroneRegistration),
    RegistrateIncident(IncidentRegistration),
}

#[derive(Clone)]
pub struct DroneRegistration {
    pub id: String,
    pub password: String,
    pub anchor_x: String,
    pub anchor_y: String,
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
