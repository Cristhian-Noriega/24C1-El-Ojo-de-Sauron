const SEPARATOR: char = ';';
const ELEMENTS_COUNT: usize = 6;

#[derive(Debug, PartialEq, Clone)]
pub struct Incident {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub state: String, // Cambiar a Enum
}

impl Incident {
    pub fn from_string(string: String) -> Result<Self, ()> {
        let splited_string: Vec<&str> = string.split(SEPARATOR).collect();

        if splited_string.len() != ELEMENTS_COUNT {
            return Err(());
        }

        let uuid = splited_string[0].to_string();
        let name = splited_string[1].to_string();
        let description = splited_string[2].to_string();
        let x_coordinate = splited_string[3].parse().unwrap();
        let y_coordinate = splited_string[4].parse().unwrap();
        let state = splited_string[5].to_string();

        Ok(Incident {
            uuid,
            name,
            description,
            x_coordinate,
            y_coordinate,
            state,
        })
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }
}
