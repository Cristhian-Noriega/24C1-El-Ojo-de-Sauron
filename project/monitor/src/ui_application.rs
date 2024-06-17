use crate::{
    camera::Camera,
    channels_tasks::{DroneRegistration, IncidentRegistration, MonitorAction, UIAction},
    drone::Drone,
    incident::Incident,
};
use eframe::egui::{Color32, FontId, Stroke};
use egui::Context;
use egui_extras::{Column, TableBuilder};
use std::sync::mpsc::{Receiver, Sender};
use walkers::{sources::OpenStreetMap, Map, MapMemory, Position, Tiles, extras::{Places, Place, Style}};

const DEFAULT_LONGITUDE: f64 = -58.3717;
const DEFAULT_LONGITUD: f64 = -34.6081;

#[derive(PartialEq)]
enum Layout {
    IncidentMap,
    IncidentList,
    NewIncident,
    DroneList,
    NewDrone,
}

pub struct UIApplication {
    new_incident_registration: IncidentRegistration,
    new_drone_registration: DroneRegistration,

    connection_status: bool,
    current_layout: Layout,
    tiles: Tiles,
    map_memory: MapMemory,

    sender: Sender<UIAction>,
    receiver: Receiver<MonitorAction>,

    drones: Vec<Drone>,
    incidents: Vec<Incident>,
    cameras: Vec<Camera>,
}

impl UIApplication {
    pub fn new(
        egui_ctx: Context,
        sender: Sender<UIAction>,
        receiver: Receiver<MonitorAction>,
    ) -> Self {

        let drone_1 = Drone {
            id: "1".to_string(),
            x_coordinate: -58.3717,
            y_coordinate: -34.6081,
            state: "Free".to_string(),
            battery: 100,
        };
        
        let drone_2 = Drone {
            id: "2".to_string(),
            x_coordinate: -58.3021,
            y_coordinate: -34.6081,
            state: "Attending Incident".to_string(),
            battery: 100,
        };

        let drone_3 = Drone {
            id: "3".to_string(),
            x_coordinate: -58.3717,
            y_coordinate: -34.6017,
            state: "Travelling".to_string(),
            battery: 100,
        };

        let drone_4 = Drone {
            id: "4".to_string(),
            x_coordinate: -58.3727,
            y_coordinate: -34.6041,
            state: "Charging".to_string(),
            battery: 100,
        };

        let incident_1 = Incident {
            uuid: "1".to_string(),
            name: "Incident 1".to_string(),
            description: "Incident 1 description".to_string(),
            x_coordinate: -58.3707,
            y_coordinate: -34.6181,
            state: "Active".to_string(),
        };

        let incident_2 = Incident {
            uuid: "2".to_string(),
            name: "Incident 2".to_string(),
            description: "Incident 2 description".to_string(),
            x_coordinate: -58.3317,
            y_coordinate: -34.6181,
            state: "Active".to_string(),
        };

        let incident_3 = Incident {
            uuid: "3".to_string(),
            name: "Incident 3".to_string(),
            description: "Incident 3 description".to_string(),
            x_coordinate: -58.0495,
            y_coordinate: -34.6000,
            state: "Active".to_string(),
        };

        let camera_1 = Camera {
            id: "1".to_string(),
            x_coordinate: -58.3517,
            y_coordinate: -45.6281,
            state: "Active".to_string(),
        };

        let camera_2 = Camera {
            id: "2".to_string(),
            x_coordinate: -58.3617,
            y_coordinate: -34.6018,
            state: "Active".to_string(),
        };

        let camera_3 = Camera {
            id: "3".to_string(),
            x_coordinate: -58.3457,
            y_coordinate: -34.6091,
            state: "Active".to_string(),
        };

        Self {
            new_incident_registration: IncidentRegistration {
                name: String::new(),
                description: String::new(),
                x: String::new(),
                y: String::new(),
            },

            new_drone_registration: DroneRegistration {
                id: String::new(),
                password: String::new(),
                anchor_x: String::new(),
                anchor_y: String::new(),
            },

            connection_status: false,
            current_layout: Layout::IncidentMap,
            tiles: Tiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),

            sender,
            receiver,
            drones: vec![drone_1, drone_2, drone_3, drone_4],
            incidents: vec![incident_1, incident_2, incident_3],
            cameras: vec![camera_1, camera_2, camera_3],
        }
    }
}

fn update_drones(drones: &mut Vec<Drone>, drone: Drone) {
    for d in drones.iter_mut() {
        if d.id == drone.id {
            *d = drone;
            return;
        }
    }

    drones.push(drone);
}

fn update_incidents(incidents: &mut Vec<Incident>, incident: Incident) {
    for i in incidents.iter_mut() {
        if i.uuid == incident.uuid {
            *i = incident;
            return;
        }
    }

    incidents.push(incident);
}

fn update_cameras(cameras: &mut Vec<Camera>, camera: Camera) {
    for c in cameras.iter_mut() {
        if c.id == camera.id {
            *c = camera;
            return;
        }
    }

    cameras.push(camera);
}

fn display_incident_map(ui: &mut egui::Ui, incidents: &Vec<Incident>, drones: &Vec<Drone>, cameras: &Vec<Camera>, tiles: &mut Tiles, map_memory: &mut MapMemory) {
    let position = Position::from_lon_lat(DEFAULT_LONGITUDE, DEFAULT_LONGITUD);
    //let map = Map::new(Some(tiles), map_memory, position);
    //ui.add(map);

    //let center_position = Position::from_lon_lat(-58.3717, -34.6081);

    let places_plugin = update_places(incidents, drones, cameras);
    
    ui.add(
        Map::new(Some(tiles), map_memory, position)
            .with_plugin(places_plugin),
    );
}

fn display_new_incident(
    ui: &mut egui::Ui,
    new_incident: &mut IncidentRegistration,
    sender: &Sender<UIAction>,
) {
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.add_space(38.0);
        ui.text_edit_singleline(&mut new_incident.name);
    });
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label("Description:");
        ui.add_space(8.0);
        ui.text_edit_multiline(&mut new_incident.description);
    });
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label("Coordenates:");
        ui.text_edit_singleline(&mut new_incident.x);
        ui.text_edit_singleline(&mut new_incident.y);
    });

    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        if ui.button("Send").clicked() {
            sender
                .send(UIAction::RegistrateIncident(new_incident.clone()))
                .unwrap();
        }
    });
}

fn display_incident_list(ui: &mut egui::Ui, incidents: &Vec<Incident>,  sender: &Sender<UIAction>) {
    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(Column::remainder(), 6)
        .header(10.0, |mut header| {
            header.col(|ui| {
                ui.heading("UUID");
            });
            header.col(|ui| {
                ui.heading("Name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|ui| {
                ui.heading("Coordinates");
            });
            header.col(|ui| {
                ui.heading("State");
            });
            header.col(|ui| { 
                ui.heading("Actions");
            });
        })
        .body(|mut body| {
            for incident in incidents.iter() {
                body.row(50.0, |mut row| {
                    row.col(|ui| {
                        ui.label(incident.uuid.clone());
                    });
                    row.col(|ui| {
                        ui.label(incident.name.clone());
                    });
                    row.col(|ui| {
                        ui.label(incident.description.clone());
                    });
                    row.col(|ui| {
                        ui.label(format!(
                            "({}, {})",
                            incident.x_coordinate, incident.y_coordinate
                        ));
                    });
                    row.col(|ui| {
                        ui.label(incident.state.clone());
                    });
                    row.col(|ui| { 
                        if ui.button("Resolve").clicked() {
                            sender.send(UIAction::ResolveIncident(incident.clone())).unwrap();
                        }
                    });
                });
            }
        });
}

fn display_new_drone(
    ui: &mut egui::Ui,
    drone_registration: &mut DroneRegistration,
    sender: &Sender<UIAction>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label("New Drone Registration:");

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Drone ID:");
            ui.add_space(22.0);
            ui.add(egui::TextEdit::singleline(&mut drone_registration.id).desired_width(340.0));
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add_space(20.0);
            ui.add(
                egui::TextEdit::singleline(&mut drone_registration.password).desired_width(340.0),
            );
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            // ui.label("Anchor Point Coordinates:");

            // ui.add_space(10.0);
            // ui.label("x:");
            // ui.add(
            //     egui::TextEdit::singleline(&mut drone_registration.anchor_x).desired_width(100.0),
            // );

            // ui.add_space(10.0);
            // ui.label("y:");
            // ui.add(
            //     egui::TextEdit::singleline(&mut drone_registration.anchor_y).desired_width(100.0),
            // );
            ui.add_space(263.0);
            if ui.button("Register").clicked() {
                sender
                    .send(UIAction::RegistrateDrone(drone_registration.clone()))
                    .unwrap();
            }
        });
    });

    ui.add_space(20.0);
}

fn display_drone_list(ui: &mut egui::Ui, drones: &Vec<Drone>) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label("Active Drones:");
        ui.add_space(10.0);
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::remainder(), 4)
            .header(10.0, |mut header| {
                header.col(|ui| {
                    ui.heading("ID");
                });
                header.col(|ui| {
                    ui.heading("Position");
                });
                header.col(|ui| {
                    ui.heading("State");
                });
                header.col(|ui| {
                    ui.heading("Battery");
                });
            })
            .body(|mut body| {
                for drone in drones.iter() {
                    body.row(50.0, |mut row| {
                        row.col(|ui| {
                            ui.label(drone.id.clone());
                        });
                        row.col(|ui| {
                            let position =
                                format!("({}, {})", drone.x_coordinate, drone.y_coordinate);
                            ui.label(position);
                        });
                        row.col(|ui| {
                            ui.label(drone.state.clone());
                        });
                        row.col(|ui| {
                            ui.label(format!("{}%", drone.battery));
                        });
                    });
                }
            });
    });
}

fn display_header(
    ui: &mut egui::Ui,
    current_layout: &mut Layout,
    sender: &Sender<UIAction>,
    conection_status: bool,
) {
    ui.horizontal(|ui| {
        ui.image(egui::include_image!("images/logo.png"));
        ui.heading(egui::RichText::new("Monitor").size(30.0));
    });
    ui.add_space(15.0);
    ui.horizontal(|ui| {
        // if ui.button("Connect").clicked() {
        //     sender.send(UIAction::Connect).unwrap();
        // }
        // ui.add_space(500.0);

        // if conection_status {
        ui.label(egui::RichText::new("Connected").color(egui::Color32::GREEN));
        // } else {
        //     ui.label(egui::RichText::new("Disconnected").color(egui::Color32::RED));
        // }
    });

    ui.add_space(20.0);

    ui.horizontal(|ui| {
        ui.selectable_value(current_layout, Layout::IncidentMap, "Map");
        ui.selectable_value(current_layout, Layout::IncidentList, "Incident List");
        ui.selectable_value(current_layout, Layout::NewIncident, "Create incident");
        ui.selectable_value(current_layout, Layout::DroneList, "Drone List");
        ui.selectable_value(current_layout, Layout::NewDrone, "Register Drone");
    });

    ui.add_space(20.0);
}

impl eframe::App for UIApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        loop {
            match self.receiver.try_recv() {
                // Ok(MonitorAction::Disconnect) => {
                //     self.conection_status = false;
                // }
                // Ok(MonitorAction::Connect) => {
                //     self.conection_status = true;
                // }
                Ok(MonitorAction::DroneData(drone)) => {
                    update_drones(&mut self.drones, drone);
                }
                Ok(MonitorAction::IncidentData(incident)) => {
                    update_incidents(&mut self.incidents, incident);
                }
                Ok(MonitorAction::CameraData(camera)) => {
                    update_cameras(&mut self.cameras, camera);
                }
                Err(_) => break,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            display_header(
                ui,
                &mut self.current_layout,
                &self.sender,
                self.connection_status,
            );

            match self.current_layout {
                Layout::IncidentMap => {
                    display_incident_map(ui, &self.incidents, &self.drones, &self.cameras, &mut self.tiles, &mut self.map_memory)
                }
                Layout::NewIncident => {
                    display_new_incident(ui, &mut self.new_incident_registration, &self.sender)
                }
                Layout::IncidentList => display_incident_list(ui, &self.incidents, &self.sender),
                Layout::DroneList => display_drone_list(ui, &self.drones),
                Layout::NewDrone => {
                    display_new_drone(ui, &mut self.new_drone_registration, &self.sender)
                }
            }
        });
    }
}

fn update_places(incidents: &Vec<Incident>, drones: &Vec<Drone>, cameras: &Vec<Camera>) -> Places{
    let mut places = vec![];

    for incident in incidents {
        let place = Place {
            position: Position::from_lon_lat(incident.x_coordinate, incident.y_coordinate),
            label: incident.name.clone(),
            symbol: 'I',
            style: Style {
                label_font: FontId::proportional(14.0),
                label_color: Color32::WHITE,
                label_background: Color32::from_rgb(255, 0, 0), // Red background
                symbol_font: FontId::monospace(18.0),
                symbol_color: Color32::from_rgb(255, 255, 255), // White symbol
                symbol_background: Color32::from_rgb(255, 0, 0), // Red background
                symbol_stroke: Stroke::new(2.0, Color32::BLACK), // Black border
            },
        };
        places.push(place);
    }

    for drone in drones {
        let color = match drone.state.as_str() {
            "Free" => Color32::from_rgb(0, 255, 0), // Green
            "Attending Incident" => Color32::from_rgb(255, 0, 0), // Red
            "Travelling" => Color32::from_rgb(255, 255, 0), // Yellow
            _ => Color32::from_rgb(0, 0, 255), // Blue
        };
        let place = Place {
            position: Position::from_lon_lat(drone.x_coordinate, drone.y_coordinate),
            label: drone.id.clone(),
            symbol: 'D',
            style: Style {
                label_font: FontId::proportional(14.0),
                label_color: Color32::WHITE,
                label_background: color, // Blue background
                symbol_font: FontId::monospace(18.0),
                symbol_color: Color32::from_rgb(255, 255, 255), // White symbol
                symbol_background: color, // Blue background
                symbol_stroke: Stroke::new(2.0, Color32::BLACK), // Black border
            },
        };
        places.push(place);
    }

    for camera in cameras {
        let place = Place {
            position: Position::from_lon_lat(camera.x_coordinate, camera.y_coordinate),
            label: camera.id.clone(),
            symbol: 'C',
            style: Style {
                label_font: FontId::proportional(14.0),
                label_color: Color32::WHITE,
                label_background: Color32::from_rgb(255, 165, 0), // Orange background
                symbol_font: FontId::monospace(18.0),
                symbol_color: Color32::from_rgb(255, 255, 255), // White symbol
                symbol_background: Color32::from_rgb(255, 165, 0), // Orange background
                symbol_stroke: Stroke::new(2.0, Color32::BLACK), // Black border
            },
        };
        places.push(place);
    }


    Places::new(places)

}