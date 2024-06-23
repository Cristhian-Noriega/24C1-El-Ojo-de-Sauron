/// Camera struct
pub struct Camera {
    pub id: String,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub status: CameraStatus,
}

pub enum CameraStatus {
    Active,
    Inactive,
}

impl CameraStatus {
    pub fn to_str(&self) -> String {
        match self {
            CameraStatus::Active => "Active".to_string(),
            CameraStatus::Inactive => "Inactive".to_string(),
        }
    }
}

impl Camera {
    /// Creates a new camera
    pub fn new(id: String, x_coordinate: f64, y_coordinate: f64, status_str: String) -> Self {
        let status = match status_str.as_str() {
            "1" => CameraStatus::Active,
            _ => CameraStatus::Inactive,
        };

        Camera {
            id,
            x_coordinate,
            y_coordinate,
            status,
        }
    }
}
