use crate::camera_status::CameraStatus;

pub struct Camera {
    id: u8,
    x_coordinate: f64,
    y_coordinate: f64,
    status: CameraStatus,
}

impl Camera {
    pub fn new(id: u8, x_coordinate: f64, y_coordinate: f64) -> Self {
        Camera {
            id,
            x_coordinate,
            y_coordinate,
            status: CameraStatus::Sleep,
        }
    }
}
