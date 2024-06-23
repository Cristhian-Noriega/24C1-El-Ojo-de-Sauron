#![allow(clippy::too_many_arguments)]

use crate::{
    channels_tasks::{
        DroneRegistration, IncidentEdit, IncidentRegistration, MonitorAction, UIAction,
    },
    camera::{Camera, CameraStatus},
    drone::{Drone, DroneStatus},
    right_click_menu::RightClickMenu,
};
use common::incident::{Incident, IncidentStatus};
use eframe::egui::{Color32, FontId, Stroke};

use eframe::egui;
use egui::{Context, Response, Ui};
use egui_extras::{Column, TableBuilder};

use std::sync::mpsc::{Receiver, Sender};
use walkers::{
    extras::{Place, Places, Style},
    sources::OpenStreetMap,
    Map, MapMemory, Position, Tiles,
};

pub const DEFAULT_LONGITUDE: f64 = -58.372170426210836;
pub const DEFAULT_LATITUDE: f64 = -34.60840997593428;

const CHARGING_STATION_LONGITUDE: f64 = -58.3703;
const CHARGING_STATION_LATITUDE: f64 = -34.6082;

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
    charging_station_coordenates: (f64, f64),

    right_click_menu: RightClickMenu,
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
            charging_station_coordenates: (CHARGING_STATION_LONGITUDE, CHARGING_STATION_LATITUDE),

            right_click_menu: RightClickMenu::default(),
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

/// Handles the right clicks in the map to open the incident registration menu with coordenates selected
fn handle_right_clicks(
    ui: &mut Ui,
    response: Response,
    right_click_menu: &mut RightClickMenu,
    map_memory: &mut MapMemory,
    new_incident_registration: &mut IncidentRegistration,
    sender: &Sender<UIAction>,
    layout: &mut Layout,
) {
    ui.ctx().input(|i| {
        if response.hovered() && i.pointer.secondary_clicked() {
            let click_location_pixels = i.pointer.hover_pos().unwrap_or_default();
            right_click_menu.update(click_location_pixels, response, map_memory);
        } else if response.hovered() && i.pointer.primary_clicked() {
            right_click_menu.open = false;
        }
    });

    if right_click_menu.open {
        egui::Area::new(right_click_menu.id)
            .fixed_pos(right_click_menu.pos_2)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    if ui.button("Register New Incident").clicked() {
                        new_incident_registration.name = String::new();
                        new_incident_registration.description = String::new();
                        new_incident_registration.x = right_click_menu.x_coordenate.to_string();
                        new_incident_registration.y = right_click_menu.y_coordenate.to_string();
                        println!(
                            "New incident at coordenates: ({}, {})",
                            right_click_menu.x_coordenate, right_click_menu.y_coordenate
                        );
                        display_new_incident(ui, new_incident_registration, sender);
                        *layout = Layout::NewIncident;

                        right_click_menu.open = false; // Close menu
                    }
                    // if ui.button("Register New Drone").clicked() {
                    //     // Handle Option 2 click
                    //     println!("New drone at at coordenates: ({}, {})", right_click_menu.x_coordenate, right_click_menu.y_coordenate);
                    //     right_click_menu.open = false; // Close menu
                    // }
                    if ui.button("Cancel").clicked() {
                        // Handle Option 2 click
                        println!("Menu closed");
                        right_click_menu.open = false; // Close menu
                    }
                });
            });
    }
}

/// Displays the incident map
fn display_incident_map(
    ui: &mut egui::Ui,
    incidents: &Vec<Incident>,
    drones: &Vec<Drone>,
    cameras: &Vec<Camera>,
    charging_station_coordenates: &(f64, f64),
    tiles: &mut Tiles,
    map_memory: &mut MapMemory,
    right_click_menu: &mut RightClickMenu,
    new_incident_registration: &mut IncidentRegistration,
    layout: &mut Layout,
    sender: &Sender<UIAction>,
) {
    let position = Position::from_lon_lat(DEFAULT_LONGITUDE, DEFAULT_LATITUDE);

    let map = Map::new(Some(tiles), map_memory, position);

    let places_plugin = update_places(incidents, drones, cameras, charging_station_coordenates);
    let map_with_plugin = map.with_plugin(places_plugin);

    let response = ui.add(map_with_plugin);

    handle_right_clicks(
        ui,
        response,
        right_click_menu,
        map_memory,
        new_incident_registration,
        sender,
        layout,
    );
}

/// Displays the form to create a new incident
fn display_new_incident(
    ui: &mut egui::Ui,
    new_incident: &mut IncidentRegistration,
    sender: &Sender<UIAction>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label("New Incident Registration:");
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.add_space(39.0);
            ui.add(egui::TextEdit::singleline(&mut new_incident.name).desired_width(400.0));
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.add_space(8.0);
            ui.add(egui::TextEdit::multiline(&mut new_incident.description).desired_width(400.0));
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("Coordenates:");
            ui.add(egui::TextEdit::singleline(&mut new_incident.x).desired_width(193.0));
            ui.add(egui::TextEdit::singleline(&mut new_incident.y).desired_width(192.0));
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(444.0);
            if ui.button("Create").clicked() {
                sender
                    .send(UIAction::RegistrateIncident(new_incident.clone()))
                    .unwrap();
                new_incident.name.clear();
                new_incident.description.clear();
                new_incident.x.clear();
                new_incident.y.clear();
            }
        });
    });
}

/// Displays the form to edit a incident
fn display_edit_incident(
    ui: &mut egui::Ui,
    edit_incident: &mut IncidentEdit,
    sender: &Sender<UIAction>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label("Incident Modifications:");
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("UUID:");
            ui.add_space(70.0);
            ui.text_edit_singleline(&mut edit_incident.uuid);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("New name:");
            ui.add_space(40.0);
            ui.text_edit_singleline(&mut edit_incident.name);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("New description:");
            ui.add_space(8.0);
            ui.text_edit_multiline(&mut edit_incident.description);
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(368.0);
            if ui.button("Edit").clicked() {
                sender
                    .send(UIAction::EditIncident(edit_incident.clone()))
                    .unwrap();
                edit_incident.uuid.clear();
                edit_incident.name.clear();
                edit_incident.description.clear();
            }
        });
    });
}

/// Displays the incident list
fn display_incident_list(
    ui: &mut egui::Ui,
    incidents: &[Incident],
    sender: &Sender<UIAction>,
    new_incident_edit: &mut IncidentEdit,
    current_layout: &mut Layout,
) {
    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(50.0)) 
        .column(Column::remainder())   
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::initial(75.0))
        .column(Column::remainder())
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
                ui.heading("Status");
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
                        ui.label(incident.status.clone().meaning());
                    });
                    row.col(|ui| {
                        if incident.status == IncidentStatus::Resolvable {
                            if ui.button("Resolve").clicked() {
                                sender
                                    .send(UIAction::ResolveIncident(incident.clone()))
                                    .unwrap();
                            }
                        } else if ui.button("Edit").clicked() {
                            new_incident_edit.uuid.clone_from(&incident.uuid);
                            new_incident_edit.name.clone_from(&incident.name);
                            new_incident_edit
                                .description
                                .clone_from(&incident.description);
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
            ui.add_space(376.0);
            if ui.button("Register").clicked() {
                sender
                    .send(UIAction::RegistrateDrone(drone_registration.clone()))
                    .unwrap();
                drone_registration.id.clear();
                drone_registration.username.clear();
                drone_registration.password.clear();
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
                            ui.label(drone.status.to_str());
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
                            ui.label(camera.status.to_str().clone());
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
) {
    ui.horizontal(|ui| {
        ui.heading(egui::RichText::new("Monitoring Aplication").size(30.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.image(egui::include_image!("images/logo.png"));
        });
    });
    ui.add_space(15.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Connected").color(egui::Color32::GREEN));
    });

    ui.add_space(20.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(current_layout, Layout::IncidentMap, "Map");
            ui.label("|");
            ui.selectable_value(current_layout, Layout::IncidentList, "Incident List");
            ui.label("|");
            ui.selectable_value(current_layout, Layout::NewIncident, "Create incident");
            ui.label("|");
            ui.selectable_value(current_layout, Layout::EditIncident, "Edit incident");
            ui.label("|");
            ui.selectable_value(current_layout, Layout::DroneList, "Drone List");
            ui.label("|");
            ui.selectable_value(current_layout, Layout::NewDrone, "Register Drone");
            ui.label("|");
            ui.selectable_value(current_layout, Layout::CameraList, "Camera List");

        });
    });

    ui.add_space(20.0);
}

impl eframe::App for UIApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        loop {
            match self.receiver.try_recv() {
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
            );

            match self.current_layout {
                Layout::IncidentMap => display_incident_map(
                    ui,
                    &self.incidents,
                    &self.drones,
                    &self.cameras,
                    &self.charging_station_coordenates,
                    &mut self.tiles,
                    &mut self.map_memory,
                    &mut self.right_click_menu,
                    &mut self.new_incident_registration,
                    &mut self.current_layout,
                    &self.sender,
                ),
                Layout::NewIncident => {
                    display_new_incident(ui, &mut self.new_incident_registration, &self.sender)
                }
                Layout::EditIncident => {
                    display_edit_incident(ui, &mut self.new_incident_edit, &self.sender)
                }
                Layout::IncidentList => display_incident_list(
                    ui,
                    &self.incidents,
                    &self.sender,
                    &mut self.new_incident_edit,
                    &mut self.current_layout,
                ),
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
fn update_places(incidents: &Vec<Incident>, drones: &Vec<Drone>, cameras: &Vec<Camera>, charging_station_coordenates: &(f64, f64)) -> Places {
    let mut places = vec![];
    
    let mut activity_cordenates = vec![];

    for incident in incidents {
        let place = Place {
            position: Position::from_lon_lat(incident.x_coordinate, incident.y_coordinate),
            label: format!("  {}",incident.name.clone()),
            symbol: '⚠', 
            style: Style {
                label_font: FontId::proportional(13.0),
                label_color: Color32::BLACK,
                label_background: Color32::TRANSPARENT, 
                symbol_font: FontId::monospace(30.0),
                symbol_color: Color32::RED,              
                symbol_background: Color32::TRANSPARENT, 
                symbol_stroke: Stroke::new(2.0, Color32::TRANSPARENT), 
            },
        };
        places.push(place);
        activity_cordenates.push((incident.x_coordinate, incident.y_coordinate));
    }
    
    for drone in drones {
        let color = match drone.status {
            DroneStatus::Free => Color32::BLACK,
            DroneStatus::AttendingIncident => Color32::from_rgb(220, 20, 60), 
            DroneStatus::Travelling => Color32::from_rgb (255, 79,0),        
            DroneStatus::Recharging => Color32::GREEN,        
        };

        if activity_cordenates.contains(&(drone.x_coordinate, drone.y_coordinate)) {
            let overlapping_activity = places.iter_mut().find(|place| {
                place.position == Position::from_lon_lat(drone.x_coordinate, drone.y_coordinate)
            }).unwrap();
            overlapping_activity.label = format!("{}, ❇{}", overlapping_activity.label, drone.id.clone());
            continue;
        }

        let place = Place {
            position: Position::from_lon_lat(drone.x_coordinate, drone.y_coordinate),
            label: format!("  {}", drone.id.clone()),
            symbol: '❇',
            style: Style {
                label_font: FontId::proportional(15.0),
                label_color: Color32::BLACK,
                label_background: Color32::TRANSPARENT, 
                symbol_font: FontId::monospace(32.0),
                symbol_color: color,                     
                symbol_background: Color32::TRANSPARENT, 
                symbol_stroke: Stroke::new(2.0, Color32::TRANSPARENT),
            },
        };
        places.push(place);
        activity_cordenates.push((drone.x_coordinate, drone.y_coordinate));
    }

    for camera in cameras {
        let color = match camera.status {
            CameraStatus::Inactive => Color32::BLACK,      
            CameraStatus::Active => Color32::RED,
        };

        let place = Place {
            position: Position::from_lon_lat(camera.x_coordinate, camera.y_coordinate),
            label: format!(" {}", camera.id.clone()),
            symbol: '📹', 
            style: Style {
                label_font: FontId::proportional(14.0),
                label_color: Color32::BLACK,
                label_background: Color32::TRANSPARENT, 
                symbol_font: FontId::monospace(28.0),
                symbol_color: color,            
                symbol_background: Color32::TRANSPARENT, 
                symbol_stroke: Stroke::new(2.0, Color32::TRANSPARENT),
            },
        };
        places.push(place);
    }

    let charging_station_place = Place {
        position: Position::from_lon_lat(charging_station_coordenates.0, charging_station_coordenates.1),
        label: "   Drone Central".to_string(),
        symbol: '🖧', // 🔋🔌🖥🖧
        style: Style {
            label_font: FontId::proportional(10.0),
            label_color: Color32::BLACK,
            label_background: Color32::TRANSPARENT, 
            symbol_font: FontId::monospace(35.0),
            symbol_color: Color32::BLACK,            
            symbol_background: Color32::TRANSPARENT, 
            symbol_stroke: Stroke::new(2.0, Color32::TRANSPARENT), 
        },
    };
    places.push(charging_station_place);

    Places::new(places)
}
