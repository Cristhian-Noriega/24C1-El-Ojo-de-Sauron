use crate::{camera::Camera, drone::Drone, incident::Incident};

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
    pub anchor_x: String,
    pub anchor_y: String,
}

impl DroneRegistration {
    pub fn build_drone_message(&self) -> String {
        format!(
            "{};{};{};{}",
            self.id, self.password, self.anchor_x, self.anchor_y
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

impl IncidentRegistration {
    pub fn build_incident_message(&self) -> String {
        format!(
            "{};{};{};{}",
            self.name, self.description, self.x, self.y
        )
    }
}

pub enum MonitorAction {
    // Connect,
    // Disconnect,
    DroneData(Drone),
    CameraData(Camera),
    IncidentData(Incident),
}
