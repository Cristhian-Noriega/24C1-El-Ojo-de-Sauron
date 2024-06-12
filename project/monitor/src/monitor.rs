use crate::{camera::Camera, client::Client, drone::Drone, incident::Incident};
use mqtt::model::packets::publish::Publish;

pub struct Monitor {
    pub incidents: Vec<Incident>,
    pub drones: Vec<Drone>,
    pub cameras: Vec<Camera>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            incidents: Vec::new(),
            drones: Vec::new(),
            cameras: Vec::new(),
        }
    }

    pub fn new_incident(
        &self,
        name: &str,
        description: &str,
        x_coordenate: &str,
        y_coordenate: &str,
        client: &Client,
    ) -> std::io::Result<()> {
        println!("Building new incident {:?}", name);

        let new_incident_topic = "new-incident";
        let x_coordenate_float: f64 = x_coordenate.parse().unwrap();
        let y_coordenate_float: f64 = y_coordenate.parse().unwrap();
        let new_incident = Incident::new(
            name.to_string(),
            description.to_string(),
            x_coordenate_float,
            y_coordenate_float,
            "Open".to_string(),
        );
        let message = new_incident.build_new_incident_message();

        client.publish(new_incident_topic, &message)?;

        // let attending_topic = format!("attending-incident/{}", new_incident.uuid);
        // let close_topic = format!("close-incident/{}", new_incident.uuid);
        // self.subscribe(&attending_topic)?;
        // self.subscribe(&close_topic)?;

        self.incidents.lock().unwrap().push(new_incident);

        Ok(())
    }

    pub fn new_drone(
        &self,
        id: &str,
        password: &str,
        x_coordenate: &str,
        y_coordenate: &str,
        client: &Client,
    ) -> std::io::Result<()> {
        let new_drone_topic = "new-drone";
        let x_coordenate_float: f64 = x_coordenate.parse().unwrap();
        let y_coordenate_float: f64 = y_coordenate.parse().unwrap();
        let message = format!(
            "{},{},{},{}",
            id, password, x_coordenate_float, y_coordenate_float
        );

        client.publish(new_drone_topic, &message)?;

        Ok(())
    }

    pub fn handle_camera_data(&self, packet: Publish) -> std::io::Result<()> {
        println!("Handling camera data {:?}", packet);
        Ok(())
    }

    pub fn handle_dron_data(&self, packet: Publish) -> std::io::Result<()> {
        println!("Handling dron data {:?}", packet);
        Ok(())
    }

    pub fn handle_attend_incident_data(&self, packet: Publish) -> std::io::Result<()> {
        println!("Handling attending incident data {:?}", packet);
        Ok(())
    }

    pub fn handle_close_incident_data(&self, packet: Publish) -> std::io::Result<()> {
        println!("Handling close incident data {:?}", packet);
        Ok(())
    }
}
