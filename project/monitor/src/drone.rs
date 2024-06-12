#[derive(Debug, PartialEq)]
pub struct Drone {
    pub id: Vec<u8>,
    pub state: String,
    pub battery: usize,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
}

impl Drone {
    pub fn new(
        id: Vec<u8>,
        state: String, //pasarlo quizas a enum
        battery: usize,
        x_coordinate: f64,
        y_coordinate: f64,
    ) -> Self {
        Self {
            id,
            state,
            battery,
            x_coordinate,
            y_coordinate,
        }
    }
}
