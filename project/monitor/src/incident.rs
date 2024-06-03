use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub struct Incident {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub state: String, // Cambiar a Enum
}

impl Incident {
    pub fn new(
        name: String,
        description: String,
        x_coordinate: f64,
        y_coordinate: f64,
        state: String,
    ) -> Self {
        let uuid = Uuid::new_v4();

        Self {
            uuid: uuid.to_string(),
            name,
            description,
            x_coordinate,
            y_coordinate,
            state,
        }
    }

    pub fn build_new_incident_message(&self) -> String {
        format!(
            "{};{};{};{};{};{}",
            self.uuid,
            self.name,
            self.description,
            self.x_coordinate,
            self.y_coordinate,
            self.state
        )
    }
}
