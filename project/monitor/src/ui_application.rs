use crate::{camera::Camera, drone::Drone, incident::Incident, monitor::Monitor};
use eframe::egui;
use egui::Context;
use egui_extras::{Column, TableBuilder};
use std::sync::mpsc::{Receiver, Sender};
use walkers::{sources::OpenStreetMap, Map, MapMemory, Position, Tiles};

const DEFAULT_LONGITUDE: f64 = -58.3717;
const DEFAULT_LONGITUD: f64 = -34.6081;

#[derive(PartialEq)]
enum Layout {
    IncidentMap,
    IncidentList,
    NewIncident,
    DroneOperations,
}

enum UIAction {
    Connect,
    Disconnect,
    RegistrateDrone(DroneRegistration),
    RegistrateIncident(IncidentRegistration),
}

struct DroneRegistration {
    id: String,
    password: String,
    anchor_x: f64,
    anchor_y: f64,
}

struct IncidentRegistration {
    name: String,
    description: String,
    x: f64,
    y: f64,
}

enum MonitorAction {
    Connect,
    Disconnect,
    DroneData(Drone),
    CameraData(Camera),
    IncidentData(Incident),
}

pub struct UIApplication {
    new_incident_name: String,
    new_incident_description: String,
    new_incident_x_coordenate: String,
    new_incident_y_coordenate: String,

    new_drone_id: String,
    new_drone_password: String,
    new_drone_anchor_x_coordenate: String,
    new_drone_anchor_y_coordenate: String,

    conection_status: bool,
    current_layout: Layout,
    tiles: Tiles,
    map_memory: MapMemory,

    sender: Sender<UIAction>,
    receiver: Receiver<MonitorAction>,

    drone_list: Vec<Drone>,
    incident_list: Vec<Incident>,
    camera_list: Vec<Camera>,
}

impl UIApplication {
    pub fn new(
        egui_ctx: Context,
        sender: Sender<UIAction>,
        receiver: Receiver<MonitorAction>,
    ) -> Self {
        Self {
            new_incident_name: "".to_owned(),
            new_incident_description: "".to_owned(),
            new_incident_x_coordenate: "".to_owned(),
            new_incident_y_coordenate: "".to_owned(),
            new_drone_id: "".to_owned(),
            new_drone_password: "".to_owned(),
            new_drone_anchor_x_coordenate: "".to_owned(),
            new_drone_anchor_y_coordenate: "".to_owned(),
            conection_status: false,
            current_layout: Layout::IncidentMap,
            tiles: Tiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),
            sender,
            receiver,
            drone_list: Vec::new(),
            incident_list: Vec::new(),
            camera_list: Vec::new(),
        }
    }
}

fn update_drone_list(drone_list: &mut Vec<Drone>, drone: Drone) {
    let mut found = false;
    for d in drone_list.iter_mut() {
        if d.id == drone.id {
            *d = drone;
            found = true;
            break;
        }
    }

    if !found {
        drone_list.push(drone);
    }
}

fn update_incident_list(incident_list: &mut Vec<Incident>, incident: Incident) {
    let mut found = false;
    for i in incident_list.iter_mut() {
        if i.uuid == incident.uuid {
            *i = incident;
            found = true;
            break;
        }
    }

    if !found {
        incident_list.push(incident);
    }
}

fn update_camera_list(camera_list: &mut Vec<Camera>, camera: Camera) {
    let mut found = false;
    for c in camera_list.iter_mut() {
        if c.id == camera.id {
            *c = camera;
            found = true;
            break;
        }
    }

    if !found {
        camera_list.push(camera);
    }
}

impl eframe::App for UIApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // leer todo lo que haya en el receiver

        loop {
            match self.receiver.try_recv() {
                Ok(action) => match action {
                    MonitorAction::Connect => {
                        self.conection_status = true;
                    }
                    MonitorAction::Disconnect => {
                        self.conection_status = false;
                    }
                    MonitorAction::DroneData(drone) => {
                        update_drone_list(&mut self.drone_list, drone);
                    }
                    MonitorAction::IncidentData(incident) => {
                        update_incident_list(&mut self.incident_list, incident);
                    }
                    MonitorAction::CameraData(camera) => {
                        update_camera_list(&mut self.camera_list, camera);
                    }
                },
                Err(_) => break,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.image(egui::include_image!("images/logo.png"));
                ui.heading(egui::RichText::new("Monitor").size(30.0));
            });
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    self.sender.send(UIAction::Connect).unwrap();
                }
                ui.add_space(500.0);

                if self.conection_status {
                    ui.label(egui::RichText::new("Connected").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Disconnected").color(egui::Color32::RED));
                }
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_layout, Layout::IncidentMap, "Map");
                ui.selectable_value(
                    &mut self.current_layout,
                    Layout::IncidentList,
                    "Incident List",
                );
                ui.selectable_value(
                    &mut self.current_layout,
                    Layout::NewIncident,
                    "Create incident",
                );
                ui.selectable_value(&mut self.current_layout, Layout::DroneOperations, "Drones");
            });

            ui.add_space(20.0);

            if self.current_layout == Layout::IncidentMap {
                ui.add(Map::new(
                    Some(&mut self.tiles),
                    &mut self.map_memory,
                    Position::from_lon_lat(DEFAULT_LONGITUDE, DEFAULT_LONGITUD),
                ));
            }

            if self.current_layout == Layout::NewIncident {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add_space(38.0);
                    ui.text_edit_singleline(&mut self.new_incident_name);
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.add_space(8.0);
                    ui.text_edit_multiline(&mut self.new_incident_description);
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("Coordenates:");
                    ui.text_edit_singleline(&mut self.new_incident_x_coordenate);
                    ui.text_edit_singleline(&mut self.new_incident_y_coordenate);
                });

                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    if ui.button("Send").clicked() {
                        let incident = IncidentRegistration {
                            name: self.new_incident_name.clone(),
                            description: self.new_incident_description.clone(),
                            x: self.new_incident_x_coordenate.parse().unwrap(),
                            y: self.new_incident_y_coordenate.parse().unwrap(),
                        };

                        self.sender
                            .send(UIAction::RegistrateIncident(incident))
                            .unwrap();
                    }
                });
            }

            if self.current_layout == Layout::IncidentList {
                let incidents = self.monitor.incident_list.lock().unwrap();
                TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .columns(Column::remainder(), 5)
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
                    })
                    .body(|mut body| {
                        for item in incidents.iter() {
                            body.row(50.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(item.uuid.clone());
                                });
                                row.col(|ui| {
                                    ui.label(item.name.clone());
                                });
                                row.col(|ui| {
                                    ui.label(item.description.clone());
                                });
                                row.col(|ui| {
                                    ui.label(format!(
                                        "({}, {})",
                                        item.x_coordinate, item.y_coordinate
                                    ));
                                });
                                row.col(|ui| {
                                    ui.label(item.state.clone());
                                });
                            });
                        }
                    });
            }

            if self.current_layout == Layout::DroneOperations {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label("New Drone Registration:");

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Drone ID:");
                        ui.add_space(22.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_drone_id).desired_width(340.0),
                        );
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add_space(20.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_drone_password)
                                .desired_width(340.0),
                        );
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Anchor Point Coordinates:");

                        ui.add_space(10.0);
                        ui.label("x:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_drone_anchor_x_coordenate)
                                .desired_width(100.0),
                        );

                        ui.add_space(10.0);
                        ui.label("y:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_drone_anchor_y_coordenate)
                                .desired_width(100.0),
                        );
                        ui.add_space(263.0);
                        if ui.button("Register").clicked() {
                            let drone = DroneRegistration {
                                id: self.new_drone_id.clone(),
                                password: self.new_drone_password.clone(),
                                anchor_x: self.new_drone_anchor_x_coordenate.parse().unwrap(),
                                anchor_y: self.new_drone_anchor_y_coordenate.parse().unwrap(),
                            };

                            self.sender.send(UIAction::RegistrateDrone(drone)).unwrap();
                        }
                    });
                });

                ui.add_space(20.0);

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label("Active Drones:");

                    ui.add_space(10.0);
                    let drones = self.monitor.drone_list.lock().unwrap();

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
                                        let position = format!(
                                            "({}, {})",
                                            drone.x_coordinate, drone.y_coordinate
                                        );
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
        });
    }
}
