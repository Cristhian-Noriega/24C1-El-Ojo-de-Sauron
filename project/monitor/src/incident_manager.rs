use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use crate::drone::Drone;
use common::incident::Incident;

//purpose: manage incidents, knows when two drones arrive to the same incident
// and starts a timer to resolve the incident. to be defined if it will be like this the sequence of solved incidents

pub struct IncidentManager {
    incidents: Mutex<HashMap<String, IncidentStatus>>,
    drones: Mutex<Vec<Arc<Mutex<Drone>>>>,
    drone_arrival_receiver: Receiver<(String, String)>
}

#[derive(Debug)]
struct IncidentStatus {
    incident: Incident,
    assigned_drones: Vec<String>,
    timer: Option<thread::JoinHandle<()>>
}

impl IncidentManager {
    pub fn new(drone_arrival_receiver: Receiver<(String, String)>) -> Self {
        IncidentManager {
            incidents: Mutex::new(HashMap::new()),
            drones: Mutex::new(Vec::new()),
            drone_arrival_receiver
        }
    }

    // //let incident_manager_thread = std::thread::spawn(move || {
    //     incident_manager.start(incident_manager_sender, monitor_receiver);
    // });

    pub fn start(&self) {
        while let Ok((incident_uuid, drone)) = self.drone_arrival_receiver.recv() {
            let mut incidents = self.incidents.lock().unwrap();
            if let Some(incident_status) = incidents.get_mut(&incident_uuid) {
                    incident_status.assigned_drones.push(drone.clone());
                    if incident_status.assigned_drones.len() == 2 {
                        let incident_uuid = incident_uuid.clone();
                        let manager_clone = self.clone();
                        // manajar timer
                        // let timer = thread::spawn(move || {
                        //     thread::sleep(Duration::from_secs(50));
                        //     manager_clone.resolve_incident(incident_uuid);
                        // });
                        // incident_status.timer = Some(timer);
                    }
                }
            }
    }

    pub fn add_drone(&self, drone: Arc<Mutex<Drone>>) {
        self.drones.lock().unwrap().push(drone);
    }

    pub fn handle_new_incident(&self, incident: Incident) {
        //
    }


    // fn resolve_incident(&self, incident_uuid: String) {
    //     let mut incidents = self.incidents.lock().unwrap();
    //     if let Some(incident_status) = incidents.remove(&incident_uuid) {
    //         for drone in incident_status.assigned_drones {
    //             let mut drone_locked = drone.lock().unwrap();
    //             drone_locked.set_incident(None);
    //         }
    //         println!("Incident {} resolved", incident_uuid);
    //     } else {
    //         println!("Incident {} not found", incident_uuid);
    //     }
    // }

    // pub fn handle_close_incident(&self, incident_uuid: &str) {
    //     self.resolve_incident(incident_uuid.to_string());
    // }

    // fn start_resolution_timer(&self, incident_uuid: &str) {
    //     let manager_clone = self.clone();
    //     thread::spawn(move || {
    //         thread::sleep(Duration::from_secs(incident.resolution_time_seconds));
    //         manager_clone.resolve_incident(incident_uuid.to_string());
    //     });
    // }
}
