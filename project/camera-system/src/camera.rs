use crate::camera_status::CameraStatus;

const ACTIVE_RANGE: f64 = 10.0;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    id: u8,
    x_coordinate: f64,
    y_coordinate: f64,
    status: CameraStatus,
    active_incidents: usize,
}

impl Camera {
    pub fn new(id: u8, x_coordinate: f64, y_coordinate: f64) -> Self {
        Camera {
            id,
            x_coordinate,
            y_coordinate,
            status: CameraStatus::Sleep,
            active_incidents: 0,
        }
    }

    pub fn data(&self) -> String {
        format!(
            "{};{};{};{}",
            self.id, self.x_coordinate, self.y_coordinate, self.status
        )
    }

    pub fn follow_incident(&mut self) {
        if self.active_incidents == 0 {
            self.activate();
        }
        self.active_incidents += 1;
    }

    pub fn unfollow_incident(&mut self) {
        self.active_incidents -= 1;
        if self.active_incidents == 0 {
            self.deactivate();
        }
    }

    fn activate(&mut self) {
        self.status = CameraStatus::Active;
    }

    fn deactivate(&mut self) {
        self.status = CameraStatus::Sleep;
    }

    pub fn is_near(&self, incident: &crate::incident::Incident) -> bool {
        let distance = euclidean_distance(
            self.x_coordinate,
            self.y_coordinate,
            incident.x_coordinate,
            incident.y_coordinate,
        );

        distance < ACTIVE_RANGE
    }
}

fn euclidean_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}
