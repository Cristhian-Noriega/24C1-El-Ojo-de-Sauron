use std::collections::HashMap;

use crate::camera::Camera;

use ::common::incident::Incident;

const SEPARATOR: &str = "|";

/// Camera system struct
#[derive(Debug)]
pub struct CameraSystem {
    cameras: Vec<Camera>,
    active_incidents: HashMap<String, Incident>,
}

impl CameraSystem {
    /// Creates a new camera system
    pub fn new() -> Self {
        CameraSystem {
            cameras: vec![],
            active_incidents: HashMap::new(),
        }
    }

    /// Adds a camera to the camera system
    pub fn add_camera(&mut self, camera: Camera) {
        self.cameras.push(camera)
    }

    /// Returns the data of the cameras in string format
    pub fn cameras_data(&self) -> String {
        let mut cameras_data = vec![];
        for camera in self.cameras.iter() {
            cameras_data.push(camera.data());
        }

        cameras_data.join(SEPARATOR)
    }

    /// Handles a new incident by changing the status of the cameras that are near
    pub fn new_incident(&mut self, incident: Incident) {
        let incident_id = incident.uuid.to_string();

        for camera in self.cameras.iter_mut() {
            if camera.is_near(&incident) {
                camera.follow_incident();
            }
        }

        self.active_incidents.insert(incident_id, incident);
    }

    /// Closes an incident by changing the status of the cameras that are near
    pub fn close_incident(&mut self, incident_id: &String) {
        let incident = self.active_incidents.get(incident_id).unwrap(); // TODO: que pasa si el incidente no estaba activo?

        for camera in &mut self.cameras {
            if camera.is_near(incident) {
                camera.unfollow_incident();
            }
        }

        self.active_incidents.remove(incident_id);
    }
}
