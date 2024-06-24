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

impl Default for CameraSystem {
    fn default() -> Self {
        Self::new()
    }
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
        let incident = match self.active_incidents.get(incident_id) {
            Some(incident) => incident,
            None => return,
        };

        for camera in &mut self.cameras {
            if camera.is_near(incident) {
                camera.unfollow_incident();
            }
        }

        self.active_incidents.remove(incident_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::common::incident::IncidentStatus;

    #[test]
    fn test_add_camera() {
        let mut camera_system = CameraSystem::new();
        let camera = Camera::new(1_u8, 1.5, 1.5, 3.0);
        camera_system.add_camera(camera);
        let camera_data = camera_system.cameras_data();
        assert_eq!(camera_data, "1;1.5;1.5;0");
    }

    #[test]
    fn test_new_incident() {
        let mut camera_system = CameraSystem::new();
        let camera = Camera::new(1_u8, 1.5, 1.5, 3.0);
        camera_system.add_camera(camera);
        let camera_data1 = camera_system.cameras_data();
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            1.0,
            1.0,
            IncidentStatus::Pending,
        );
        camera_system.new_incident(incident.clone());
        let camera_data2 = camera_system.cameras_data();
        assert_eq!(camera_data1, "1;1.5;1.5;0");
        assert_eq!(camera_data2, "1;1.5;1.5;1");
    }

    #[test]
    fn test_close_incident() {
        let mut camera_system = CameraSystem::new();
        let camera = Camera::new(1_u8, 1.5, 1.5, 3.0);
        camera_system.add_camera(camera);
        let camera_data1 = camera_system.cameras_data();
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            1.0,
            1.0,
            IncidentStatus::Pending,
        );
        camera_system.new_incident(incident.clone());
        let camera_data2 = camera_system.cameras_data();
        camera_system.close_incident(&incident.uuid);
        let camera_data3 = camera_system.cameras_data();
        assert_eq!(camera_data1, "1;1.5;1.5;0");
        assert_eq!(camera_data2, "1;1.5;1.5;1");
        assert_eq!(camera_data3, "1;1.5;1.5;0");
    }
}
