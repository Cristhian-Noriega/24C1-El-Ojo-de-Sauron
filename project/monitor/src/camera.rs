/// Camera struct
pub struct Camera {
    pub id: String,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub state: String,
}

impl Camera {
    /// Creates a new camera
    pub fn new(id: String, x_coordinate: f64, y_coordinate: f64, state: String) -> Self {
        Camera {
            id,
            x_coordinate,
            y_coordinate,
            state,
        }
    }
}
