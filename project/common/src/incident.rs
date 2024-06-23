use std::fmt;

use crate::error::Error;

pub const SEPARATOR: char = ';';
const ELEMENTS_COUNT: usize = 6;

/// Represents the different statuses an incident can have
#[derive(Debug, PartialEq, Clone)]
pub enum IncidentStatus {
    Pending,
    InProgress,
    Resolvable,
    Resolved,
}

impl IncidentStatus {
    /// Creates a new incident status from a string
    pub fn from_string(string: String) -> Self {
        match string.as_str() {
            "0" => IncidentStatus::Pending,
            "1" => IncidentStatus::InProgress,
            "2" => IncidentStatus::Resolvable,
            "3" => IncidentStatus::Resolved,
            _ => panic!("Invalid incident status"),
        }
    }

    /// Returns the meaning of the incident status in string format
    pub fn meaning(&self) -> String {
        match self {
            IncidentStatus::Pending => "Pending".to_string(),
            IncidentStatus::InProgress => "In Progress".to_string(),
            IncidentStatus::Resolvable => "Resolvable".to_string(),
            IncidentStatus::Resolved => "Resolved".to_string(),
        }
    }
}

impl fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status = match self {
            IncidentStatus::Pending => "0",
            IncidentStatus::InProgress => "1",
            IncidentStatus::Resolvable => "2",
            IncidentStatus::Resolved => "3",
        };

        write!(f, "{}", status)
    }
}

/// Represents an incident
#[derive(Debug, PartialEq, Clone)]
pub struct Incident {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub x_coordinate: f64,
    pub y_coordinate: f64,
    pub status: IncidentStatus,
}

impl Incident {
    /// Creates a new incident
    pub fn new(
        uuid: String,
        name: String,
        description: String,
        x_coordinate: f64,
        y_coordinate: f64,
        status: IncidentStatus,
    ) -> Self {
        Incident {
            uuid,
            name,
            description,
            x_coordinate,
            y_coordinate,
            status,
        }
    }

    /// Creates a new incident from a string
    pub fn from_string(string: String) -> Result<Self, Error> {
        let splited_string: Vec<&str> = string.split(SEPARATOR).collect();

        if splited_string.len() != ELEMENTS_COUNT {
            return Err(Error::new("Invalid incident string".to_string()));
        }

        let id = splited_string[0].to_string();
        let name = splited_string[1].to_string();
        let description = splited_string[2].to_string();
        let x_coordinate = splited_string[3].parse().unwrap();
        let y_coordinate = splited_string[4].parse().unwrap();
        let state = IncidentStatus::from_string(splited_string[5].to_string());

        Ok(Incident {
            uuid: id,
            name,
            description,
            x_coordinate,
            y_coordinate,
            status: state,
        })
    }

    /// Sets the status of the incident
    pub fn set_status(&mut self, status: IncidentStatus) {
        self.status = status;
    }

    /// Changes the name of the incident
    pub fn change_incident_name(&mut self, name: String) {
        self.name = name;
    }

    /// Changes the description of the incident
    pub fn change_incident_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn id(&self) -> String {
        self.uuid.clone()
    }
}

impl fmt::Display for Incident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{};{};{};{};{};{}",
            self.uuid,
            self.name,
            self.description,
            self.x_coordinate,
            self.y_coordinate,
            self.status
        )
    }
}
