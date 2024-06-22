use crate::{
    camera::Camera,
    channels_tasks::{DroneRegistration, IncidentRegistration, IncidentEdit, MonitorAction, UIAction},
    drone::Drone,
};
use common::incident::{Incident, IncidentStatus};
use eframe::egui::{Color32, FontId, Stroke};

use eframe::egui;
use egui::Context;
use egui_extras::{Column, TableBuilder};
use std::sync::mpsc::{Receiver, Sender};
use walkers::{
    extras::{Place, Places, Style},
    sources::OpenStreetMap,
    Map, MapMemory, Position, Tiles,
};

const DEFAULT_LONGITUDE: f64 = -58.3717;
const DEFAULT_LONGITUD: f64 = -34.6081;

/// Represents the layout of the UI
#[derive(PartialEq)]
enum Layout {
    IncidentMap,
    IncidentList,
    NewIncident,
    EditIncident,
    DroneList,
    NewDrone,
    CameraList,
}

/// Represents the UI application
pub struct UIApplication {
    new_incident_registration: IncidentRegistration,
    new_drone_registration: DroneRegistration,
    new_incident_edit: IncidentEdit,

    //connection_status: bool,
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
    /// Creates a new UI application
    pub fn new(
        egui_ctx: Context,
        sender: Sender<UIAction>,
        receiver: Receiver<MonitorAction>,
    ) -> Self {
        Self {
            new_incident_registration: IncidentRegistration {
                name: String::new(),
                description: String::new(),
                x: String::new(),
                y: String::new(),
            },

            new_drone_registration: DroneRegistration {
                id: String::new(),
                username: String::new(),
                password: String::new(),
            },

            new_incident_edit: IncidentEdit {
                uuid: String::new(),
                name: String::new(),
                description: String::new(),
            },

            //connection_status: false,
            current_layout: Layout::IncidentMap,
            tiles: Tiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),

            sender,
            receiver,
            drones: vec![],
            incidents: vec![],
            cameras: vec![],
        }
    }
}

/// Updates the drones in the UI
fn update_drones(drones: &mut Vec<Drone>, drone: Drone) {
    for d in drones.iter_mut() {
        if d.id == drone.id {
            *d = drone;
            return;
        }
    }

    drones.push(drone);
}

/// Updates the incidents in the UI
fn update_incidents(incidents: &mut Vec<Incident>, incident: Incident) {
    for i in incidents.iter_mut() {
        if i.uuid == incident.uuid {
            *i = incident;
            return;
        }
    }

    incidents.push(incident);
}

/// Updates the cameras in the UI
fn update_cameras(cameras: &mut Vec<Camera>, camera: Camera) {
    for c in cameras.iter_mut() {
        if c.id == camera.id {
            *c = camera;
            return;
        }
    }

    cameras.push(camera);
}

/// Displays the incident map
fn display_incident_map(
    ui: &mut egui::Ui,
    incidents: &Vec<Incident>,
    drones: &Vec<Drone>,
    cameras: &Vec<Camera>,
    tiles: &mut Tiles,
    map_memory: &mut MapMemory,
) {
    let position = Position::from_lon_lat(DEFAULT_LONGITUDE, DEFAULT_LONGITUD);
    // let map = Map::new(Some(tiles), map_memory, position);
    // ui.add(map);

    //let center_position = Position::from_lon_lat(-58.3717, -34.6081);

    let places_plugin = update_places(incidents, drones, cameras);

    ui.add(Map::new(Some(tiles), map_memory, position).with_plugin(places_plugin));
}

/// Displays the form to create a new incident
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

/// Displays the form to edit a incident
fn display_edit_incident(
    ui: &mut egui::Ui,
    edit_incident: &mut IncidentEdit,
    sender: &Sender<UIAction>,
) {
    ui.horizontal(|ui| {
        ui.label("UUID:");
        ui.add_space(38.0);
        ui.text_edit_singleline(&mut edit_incident.uuid);
    });
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label("New name:");
        ui.add_space(38.0);
        ui.text_edit_singleline(&mut edit_incident.name);
    });
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label("New description:");
        ui.add_space(8.0);
        ui.text_edit_multiline(&mut edit_incident.description);
    });
    ui.add_space(5.0);

    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        if ui.button("Edit").clicked() {
            sender
                .send(UIAction::EditIncident(edit_incident.clone()))
                .unwrap();
        }
    });
}

/// Displays the incident list
fn display_incident_list(ui: &mut egui::Ui, incidents: &[Incident], sender: &Sender<UIAction>, new_incident_edit: &mut IncidentEdit, current_layout: &mut Layout) {
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
                        ui.label(incident.status.clone().to_string());
                    });
                    row.col(|ui| {
                        // if incident status == Solvable -> show resolve button
                        // // otherwise show it disabled
                        // if ui.button("Resolve").clicked() {
                        //     sender
                        //         .send(UIAction::ResolveIncident(incident.clone()))
                        //         .unwrap();
                        // }

                        if incident.status == IncidentStatus::Resolvable {
                            if ui.button("Resolve").clicked() {
                                sender
                                    .send(UIAction::ResolveIncident(incident.clone()))
                                    .unwrap();
                            }
                        } else if ui.button("Edit").clicked() {
                            new_incident_edit.uuid.clone_from(&incident.uuid);
                            new_incident_edit.name.clone_from(&incident.name);
                            new_incident_edit.description.clone_from(&incident.description);
                            *current_layout = Layout::EditIncident;
                        }
                    });
                });
            }
        });
}

/// Displays the form to register a new drone
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
            ui.label("Username:");
            ui.add_space(17.0);
            ui.add(
                egui::TextEdit::singleline(&mut drone_registration.username).desired_width(340.0),
            );
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

/// Displays the drone list
fn display_drone_list(ui: &mut egui::Ui, drones: &[Drone]) {
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

/// Displays the camera list
fn display_camera_list(ui: &mut egui::Ui, cameras: &[Camera]) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label("Camera List");
        ui.add_space(10.0);
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::remainder(), 3)
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
            })
            .body(|mut body| {
                for camera in cameras.iter() {
                    body.row(50.0, |mut row| {
                        row.col(|ui| {
                            ui.label(camera.id.clone());
                        });
                        row.col(|ui| {
                            let position =
                                format!("({}, {})", camera.x_coordinate, camera.y_coordinate);
                            ui.label(position);
                        });
                        row.col(|ui| {
                            ui.label(camera.state.clone());
                        });
                    });
                }
            });
    });
}

/// Displays the header of the UI
fn display_header(
    ui: &mut egui::Ui,
    current_layout: &mut Layout,
    // sender: &Sender<UIAction>,
    // conection_status: bool,
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
        ui.selectable_value(current_layout, Layout::EditIncident, "Edit incident");
        ui.selectable_value(current_layout, Layout::DroneList, "Drone List");
        ui.selectable_value(current_layout, Layout::NewDrone, "Register Drone");
        ui.selectable_value(current_layout, Layout::CameraList, "Camera List");
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
                Ok(MonitorAction::Drone(drone)) => {
                    update_drones(&mut self.drones, drone);
                }
                Ok(MonitorAction::Incident(incident)) => {
                    update_incidents(&mut self.incidents, incident);
                }
                Ok(MonitorAction::Camera(camera)) => {
                    update_cameras(&mut self.cameras, camera);
                }
                Err(_) => break,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            display_header(
                ui,
                &mut self.current_layout,
                // &self.sender,
                // self.connection_status,
            );

            match self.current_layout {
                Layout::IncidentMap => display_incident_map(
                    ui,
                    &self.incidents,
                    &self.drones,
                    &self.cameras,
                    &mut self.tiles,
                    &mut self.map_memory,
                ),
                Layout::NewIncident => {
                    display_new_incident(ui, &mut self.new_incident_registration, &self.sender)
                }
                Layout::EditIncident => {
                    display_edit_incident(ui, &mut self.new_incident_edit, &self.sender)
                }
                Layout::IncidentList => display_incident_list(ui, &self.incidents, &self.sender, &mut self.new_incident_edit, &mut self.current_layout),
                Layout::DroneList => display_drone_list(ui, &self.drones),
                Layout::NewDrone => {
                    display_new_drone(ui, &mut self.new_drone_registration, &self.sender)
                }
                Layout::CameraList => display_camera_list(ui, &self.cameras),
            }
        });
    }
}

/// Updates the places in the map
fn update_places(incidents: &Vec<Incident>, drones: &Vec<Drone>, cameras: &Vec<Camera>) -> Places {
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
            "Free" => Color32::from_rgb(0, 255, 0),               // Green
            "Attending Incident" => Color32::from_rgb(255, 0, 0), // Red
            "Travelling" => Color32::from_rgb(255, 255, 0),       // Yellow
            _ => Color32::from_rgb(0, 0, 255),                    // Blue
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
                symbol_background: color,                       // Blue background
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
