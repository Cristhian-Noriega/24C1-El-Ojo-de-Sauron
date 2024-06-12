#[derive(Debug, PartialEq)]
pub struct Drone {
    pub id: String,
    pub password: String,
    pub state: String,
    pub battery: f64,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
}

impl Drone {
    // pub fn new(
    //     id: String,
    //     password: String,
    //     state: String,
    //     battery: f64,
    //     x_coordinate: f64,
    //     y_coordinate: f64,
    // ) -> Self {
    //     Self {
    //         id,
    //         password,
    //         state,
    //         battery,
    //         x_coordinate,
    //         y_coordinate,
    //     }
    // }

    // pub fn build_new_drone_message(&self) -> String {
    //     format!(
    //         "{};{};{};{};{};{}",
    //         self.id,
    //         self.password,
    //         self.state,
    //         self.battery,
    //         self.x_coordinate,
    //         self.y_coordinate,
    //     )
    // }
}
