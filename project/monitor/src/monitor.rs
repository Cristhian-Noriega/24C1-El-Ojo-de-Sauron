use std::collections::HashMap;

use common::incident::{Incident, IncidentStatus};

use crate::drone::Drone;

pub struct Monitor {
    drones: HashMap<String, Drone>,
    incidents: HashMap<String, Incident>,
    open_incidents: HashMap<String, usize>,
    active_incidents: HashMap<String, usize>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            drones: HashMap::new(),
            incidents: HashMap::new(),
            open_incidents: HashMap::new(),
            active_incidents: HashMap::new(),
        }
    }

    pub fn new_incident(&mut self, incident: Incident) {
        self.incidents
            .insert(incident.uuid.clone(), incident.clone());
        self.open_incidents.insert(incident.uuid.clone(), 0);
    }

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

    pub fn ready_incident(&mut self, incident_uuid: String) -> Option<Incident> {
        if let Some(incident) = self.incidents.get_mut(&incident_uuid) {
            if let Some(active_count) = self.active_incidents.get_mut(&incident_uuid) {
                *active_count -= 1;
                if *active_count == 0 {
                    self.active_incidents.remove(&incident_uuid);
                    incident.status = IncidentStatus::Resolvable;
                }
                return Some(incident.clone());
            }
        }

        None
    }

    pub fn get_incident(&self, incident_uuid: &str) -> Option<&Incident> {
        self.incidents.get(incident_uuid)
    }

    pub fn add_drone(&mut self, drone: Drone) {
        self.drones.insert(drone.id.clone(), drone);
    }

    pub fn get_drone(&self, drone_id: &str) -> Option<&Drone> {
        self.drones.get(drone_id)
    }
}
