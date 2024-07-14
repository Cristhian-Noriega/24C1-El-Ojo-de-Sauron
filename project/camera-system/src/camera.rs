use std::collections::HashSet;

use common::incident::Incident;

use common::camera_status::CameraStatus;

/// Represents a camera in the camera system
#[derive(Clone, Debug)]
pub struct Camera {
    id: u8,
    x_coordinate: f64,
    y_coordinate: f64,
    active_range: f64,
    status: CameraStatus,
    active_incidents: usize,
    seen_images: HashSet<String>,
}

impl Camera {
    /// Creates a new camera
    pub fn new(id: u8, x_coordinate: f64, y_coordinate: f64, active_range: f64) -> Self {
        Camera {
            id,
            x_coordinate,
            y_coordinate,
            active_range,
            status: CameraStatus::Sleep,
            active_incidents: 0,
            seen_images: HashSet::new(),
        }
    }

    /// Returns the id of the camera
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Returns the data of the camera in string format
    pub fn data(&self) -> String {
        format!(
            "{};{};{};{}",
            self.id, self.x_coordinate, self.y_coordinate, self.status
        )
    }

    /// Returns the position of the camera as String
    pub fn position(&self) -> String {
        format!("{};{}", self.x_coordinate, self.y_coordinate)
    }

    /// Increases the number of active incidents followed by the camera
    pub fn follow_incident(&mut self) {
        if self.active_incidents == 0 {
            self.activate();
        }
        self.active_incidents += 1;
    }

    /// Decreases the number of active incidents followed by the camera
    pub fn unfollow_incident(&mut self) {
        self.active_incidents -= 1;
        if self.active_incidents == 0 {
            self.deactivate();
        }
    }

    /// Changes the status of the camera to active
    fn activate(&mut self) {
        self.status = CameraStatus::Active;
    }

    /// Changes the status of the camera to sleep
    fn deactivate(&mut self) {
        self.status = CameraStatus::Sleep;
    }

    /// Returns true if the camera is near the incident
    pub fn is_near(&self, incident: &Incident) -> bool {
        let distance = euclidean_distance(
            self.x_coordinate,
            self.y_coordinate,
            incident.x_coordinate,
            incident.y_coordinate,
        );

        distance < self.active_range
    }

    /// Return true if the camera is sleeping
    pub fn is_sleeping(&self) -> bool {
        self.status == CameraStatus::Sleep
    }

    /// Returns true if the camera has already seen the image
    pub fn has_already_seen(&self, image: &str) -> bool {
        self.seen_images.contains(image)
    }

    /// Adds an image to the list of seen images
    pub fn add_seen_image(&mut self, image: &str) {
        self.seen_images.insert(image.to_string());
    }
}

/// Calculates the euclidean distance between two points
fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::incident::IncidentStatus;

    #[test]
    fn test_new_camera() {
        let camera = Camera::new(1, 1.5, 1.5, 3.0);
        assert_eq!(camera.id, 1);
        assert_eq!(camera.x_coordinate, 1.5);
        assert_eq!(camera.y_coordinate, 1.5);
        assert_eq!(camera.active_range, 3.0);
        assert_eq!(camera.status, CameraStatus::Sleep);
        assert_eq!(camera.active_incidents, 0);
    }

    #[test]
    fn test_data() {
        let camera = Camera::new(1, 1.5, 1.5, 3.0);
        assert_eq!(camera.data(), "1;1.5;1.5;0");
    }

    #[test]
    fn test_follow_incident() {
        let mut camera = Camera::new(1, 1.5, 1.5, 3.0);
        camera.follow_incident();
        assert_eq!(camera.active_incidents, 1);
        assert_eq!(camera.status, CameraStatus::Active);
    }

    #[test]
    fn test_unfollow_incident() {
        let mut camera = Camera::new(1, 1.5, 1.5, 3.0);
        camera.follow_incident();
        camera.unfollow_incident();
        assert_eq!(camera.active_incidents, 0);
        assert_eq!(camera.status, CameraStatus::Sleep);
    }

    #[test]
    fn test_is_near() {
        let camera = Camera::new(1, 1.5, 1.5, 3.0);
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            1.0,
            1.0,
            IncidentStatus::Pending,
        );
        assert!(camera.is_near(&incident));
    }

    #[test]
    fn test_is_not_near() {
        let camera = Camera::new(1, 1.5, 1.5, 3.0);
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            10.0,
            10.0,
            IncidentStatus::Pending,
        );
        assert!(!camera.is_near(&incident));
    }
}
