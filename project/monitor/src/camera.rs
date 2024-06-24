/// Camera struct
use common::camera_status::CameraStatus;
pub struct Camera {
    pub id: String,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub status: CameraStatus,
}

impl Camera {
    /// Creates a new camera
    pub fn new(id: String, x_coordinate: f64, y_coordinate: f64, status_str: String) -> Self {
        let status = match status_str.as_str() {
            "1" => CameraStatus::Active,
            _ => CameraStatus::Sleep,
        };

        Camera {
            id,
            x_coordinate,
            y_coordinate,
            status,
        }
    }
}
