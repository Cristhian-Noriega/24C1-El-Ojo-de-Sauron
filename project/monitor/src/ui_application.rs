use crate::{camera::Camera, drone::Drone, incident::Incident};
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
    DroneList,
    NewDrone,
}

enum UIAction {
    Connect,
    Disconnect,
    RegistrateDrone(DroneRegistration),
    RegistrateIncident(IncidentRegistration),
}

#[derive(Clone)]
struct DroneRegistration {
    id: String,
    password: String,
    anchor_x: String,
    anchor_y: String,
}

#[derive(Clone)]
struct IncidentRegistration {
    name: String,
    description: String,
    x: String,
    y: String,
}

enum MonitorAction {
    Connect,
    Disconnect,
    DroneData(Drone),
    CameraData(Camera),
    IncidentData(Incident),
}

pub struct UIApplication {
    new_incident_registration: IncidentRegistration,
    new_drone_registration: DroneRegistration,

    conection_status: bool,
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

            conection_status: false,
            current_layout: Layout::IncidentMap,
            tiles: Tiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),

            sender,
            receiver,
            drones: Vec::new(),
            incidents: Vec::new(),
            cameras: Vec::new(),
        }
    }
}

fn update_drones(drones: &mut Vec<Drone>, drone: Drone) {
    let mut found = false;
    for d in drones.iter_mut() {
        if d.id == drone.id {
            *d = drone;
            found = true;
            break;
        }
    }

    if !found {
        drones.push(drone);
    }
}

fn update_incidents(incidents: &mut Vec<Incident>, incident: Incident) {
    let mut found = false;
    for i in incidents.iter_mut() {
        if i.uuid == incident.uuid {
            *i = incident;
            found = true;
            break;
        }
    }

    if !found {
        incidents.push(incident);
    }
}

fn update_cameras(cameras: &mut Vec<Camera>, camera: Camera) {
    let mut found = false;
    for c in cameras.iter_mut() {
        if c.id == camera.id {
            *c = camera;
            found = true;
            break;
        }
    }

    if !found {
        cameras.push(camera);
    }
}

fn display_incident_map(ui: &mut egui::Ui, tiles: &mut Tiles, map_memory: &mut MapMemory) {
    let position = Position::from_lon_lat(DEFAULT_LONGITUDE, DEFAULT_LONGITUD);
    let mut map = Map::new(Some(tiles), map_memory, position);
    ui.add(map);
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

fn display_incident_list(ui: &mut egui::Ui, incidents: &Vec<Incident>) {
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
            ui.label("Anchor Point Coordinates:");

            ui.add_space(10.0);
            ui.label("x:");
            ui.add(
                egui::TextEdit::singleline(&mut drone_registration.anchor_x).desired_width(100.0),
            );

            ui.add_space(10.0);
            ui.label("y:");
            ui.add(
                egui::TextEdit::singleline(&mut drone_registration.anchor_y).desired_width(100.0),
            );
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
        if ui.button("Connect").clicked() {
            sender.send(UIAction::Connect).unwrap();
        }
        ui.add_space(500.0);

        if conection_status {
            ui.label(egui::RichText::new("Connected").color(egui::Color32::GREEN));
        } else {
            ui.label(egui::RichText::new("Disconnected").color(egui::Color32::RED));
        }
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
                Ok(MonitorAction::Disconnect) => {
                    self.conection_status = false;
                }
                Ok(MonitorAction::Connect) => {
                    self.conection_status = true;
                }
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
                self.conection_status,
            );

            match self.current_layout {
                Layout::IncidentMap => {
                    display_incident_map(ui, &mut self.tiles, &mut self.map_memory)
                }
                Layout::NewIncident => {
                    display_new_incident(ui, &mut self.new_incident_registration, &self.sender)
                }
                Layout::IncidentList => display_incident_list(ui, &self.incidents),
                Layout::DroneList => display_drone_list(ui, &self.drones),
                Layout::NewDrone => {
                    display_new_drone(ui, &mut self.new_drone_registration, &self.sender)
                }
            }
        });
    }
}
