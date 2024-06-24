use std::collections::HashMap;

use common::incident::{Incident, IncidentStatus};

/// Represents the monitor that will be handling all incidents
pub struct Monitor {
    incidents: HashMap<String, Incident>,
    open_incidents: HashMap<String, usize>,
    active_incidents: HashMap<String, usize>,
}

impl Default for Monitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Monitor {
    /// Creates a new monitor
    pub fn new() -> Self {
        Self {
            incidents: HashMap::new(),
            open_incidents: HashMap::new(),
            active_incidents: HashMap::new(),
        }
    }

    /// Registers a new incident
    pub fn new_incident(&mut self, incident: Incident) {
        self.incidents
            .insert(incident.uuid.clone(), incident.clone());
        self.open_incidents.insert(incident.uuid.clone(), 0);
    }

    /// Changes the status of an incident to in progress
    pub fn attend_incident(&mut self, incident_uuid: String) -> Option<Incident> {
        if let Some(incident) = self.incidents.get_mut(&incident_uuid) {
            if let Some(open_count) = self.open_incidents.get_mut(&incident_uuid) {
                *open_count += 1;
                if *open_count == 2 {
                    self.active_incidents
                        .insert(incident_uuid.clone(), *open_count);
                    self.open_incidents.remove(&incident_uuid);
                    incident.status = IncidentStatus::InProgress;
                }
                return Some(incident.clone());
            }
        }

        None
    }

    /// Changes the name and description of an incident
    pub fn edit_incident(
        &mut self,
        incident_uuid: String,
        name: String,
        description: String,
    ) -> Option<Incident> {
        if let Some(incident) = self.incidents.get_mut(&incident_uuid) {
            incident.name = name;
            incident.description = description;
            return Some(incident.clone());
        }
        None
    }

    /// Gets the incident by its UUID
    pub fn get_incident(&self, incident_uuid: &str) -> Option<&Incident> {
        self.incidents.get(incident_uuid)
    }

    /// Sets the incident as resolvable
    pub fn set_resolvable_incident(&mut self, incident_uuid: String) {
        if let Some(incident) = self.incidents.get_mut(&incident_uuid) {
            incident.status = IncidentStatus::Resolvable;
            self.active_incidents.insert(incident_uuid.clone(), 1);
        }
    }

    /// Sets the incident as resolved
    pub fn set_resolved_incident(&mut self, incident_uuid: String) {
        if let Some(incident) = self.incidents.get_mut(&incident_uuid) {
            incident.status = IncidentStatus::Resolved;
            self.active_incidents.remove(&incident_uuid);
        }
    }

    /// Gets the amount of incidents
    pub fn get_amount_incidents(&self) -> usize {
        self.incidents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_incident() {
        let mut monitor = Monitor::new();
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            1.0,
            1.0,
            IncidentStatus::Pending,
        );

        monitor.new_incident(incident.clone());
        assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);
    }

    #[test]
    fn test_attend_incident() {
        let mut monitor = Monitor::new();
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            1.0,
            1.0,
            IncidentStatus::Pending,
        );

        monitor.new_incident(incident.clone());
        assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

        let incident = monitor.attend_incident(incident.uuid).unwrap();
        let incident = monitor.attend_incident(incident.uuid).unwrap();

        assert_eq!(incident.status, IncidentStatus::InProgress);
    }

    #[test]
    fn test_resolvable_incident() {
        let mut monitor = Monitor::new();
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            1.0,
            1.0,
            IncidentStatus::Pending,
        );

        monitor.new_incident(incident.clone());
        assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

        monitor.set_resolvable_incident(incident.uuid.clone());
        let incident = monitor.get_incident(&incident.uuid).unwrap();
        assert_eq!(incident.status, IncidentStatus::Resolvable);
    }

    #[test]
    fn test_solve_incident() {
        let mut monitor = Monitor::new();
        let incident = Incident::new(
            "incident1".to_string(),
            "incident1".to_string(),
            "incident1".to_string(),
            1.0,
            1.0,
            IncidentStatus::Pending,
        );

        monitor.new_incident(incident.clone());
        assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

        monitor.set_resolved_incident(incident.uuid.clone());
        let incident = monitor.get_incident(&incident.uuid).unwrap();
        assert_eq!(incident.status, IncidentStatus::Resolved);
    }
}
