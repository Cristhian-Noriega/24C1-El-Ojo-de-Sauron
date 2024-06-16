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
        &mut self,
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
        let message = new_incident.build_incident_message();

        client.publish(new_incident_topic, &message)?;

        // let attending_topic = format!("attending-incident/{}", new_incident.uuid);
        // let close_topic = format!("close-incident/{}", new_incident.uuid);
        // self.subscribe(&attending_topic)?;
        // self.subscribe(&close_topic)?;

        self.incidents.push(new_incident);

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

    pub fn handle_drone_data(&self, packet: Publish) -> std::io::Result<()> {
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

    pub fn has_registered_drone(&self, id: &String) -> bool {
        self.drones.iter().any(|drone| &drone.id == id)
    }

    pub fn add_drone(&mut self, drone: Drone) {
        self.drones.push(drone);
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.cameras.push(camera);
    }

    pub fn add_incident(&mut self, incident: Incident) {
        self.incidents.push(incident);
    }

    pub fn update_drone(
        &mut self,
        id: &String,
        state: String,
        battery: usize,
        x_coordinate: f64,
        y_coordinate: f64,
    ) {
        if let Some(drone) = self.drones.iter_mut().find(|drone| &drone.id == id) {
            drone.state = state;
            drone.battery = battery;
            drone.x_coordinate = x_coordinate;
            drone.y_coordinate = y_coordinate;
        }
    }

    pub fn get_drone(&self, id: &String) -> Option<&Drone> {
        self.drones.iter().find(|drone| &drone.id == id)
    }
}
