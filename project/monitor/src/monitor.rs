use std::collections::HashMap;

use common::incident::Incident;

pub struct Monitor {
    pub active_incidents: HashMap<String, Incident>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            active_incidents: HashMap::new(),
        }
    }

    pub fn new_incident(&mut self, incident: Incident) -> std::io::Result<()> {
        self.active_incidents
            .insert(incident.uuid.clone(), incident.clone());

        Ok(())
    }
}
