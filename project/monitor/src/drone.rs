use common::incident::Incident;


#[derive(Debug, PartialEq, Clone)]
pub struct Drone {
    pub id: String,
    pub state: String,
    pub battery: usize,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub incident: Option<Incident>,
}

impl Drone {
    pub fn new(
        id: String,
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
            incident: None,
        }
    }

    pub fn is_free(&self) -> bool {
        self.state == "Free"
    }

    pub fn set_incident(&mut self, incident: Option<Incident>) {
        self.incident = incident;
    }
}
