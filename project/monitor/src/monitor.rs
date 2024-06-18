use std::{collections::HashMap, os::unix::thread, sync::{Arc, Mutex}, thread::{sleep, Thread}, time::{Duration,Instant}};

use common::incident::{Incident, IncidentStatus};

use crate::drone::Drone;

pub struct Monitor {
    incidents: HashMap<String, Incident>,
    open_incidents: HashMap<String, usize>, 
    active_incidents: HashMap<String, usize>, 
}

impl Monitor {
    pub fn new() -> Self {
        Self {
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

    pub fn is_incident_active(&self, incident_uuid: &str) -> bool {
        self.active_incidents.contains_key(incident_uuid)
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


    //cuenta regresiva para la resolucion de un incidente
    fn simulate_resolution(&self, incident_uuid: String, resolution_time: Duration) {
        let incidents = self.incidents.clone();
        let active_incidents = self.active_incidents.clone();
        //arc mutex
        let locked_incidents = Arc::new(Mutex::new(incidents));
        let locked_active_incidents: Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(active_incidents));
        
        std::thread::spawn(move || {
            let start = Instant::now();
            while start.elapsed() < resolution_time {
                std::thread::sleep(Duration::from_secs(60)); 
            }
            let mut incidents = locked_incidents.lock().unwrap();
            if let Some(incident) = incidents.get_mut(&incident_uuid) {
                incident.status = IncidentStatus::Resolvable;
            }
            locked_active_incidents.lock().unwrap().remove(&incident_uuid);
            println!("Incident {} resolved", incident_uuid);
        });
    }
}
