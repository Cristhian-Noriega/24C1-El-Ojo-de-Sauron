use crate::{client::Client, drone::Drone, incident::Incident};
use eframe::egui::{Color32, FontId, Stroke};
use egui::Context;
use egui_extras::{Column, TableBuilder};
use std::{sync::{mpsc, Arc, Mutex, MutexGuard}, vec};
use walkers::{sources::OpenStreetMap, Map, MapMemory, Position, Tiles, extras::{Places, Place, Style}};

#[derive(PartialEq)]
enum Layout {
    IncidentMap,
    IncidentList,
    NewIncident,
    DroneOperations,
}

pub struct UIApplication {
    client: Arc<Mutex<Client>>,
    
    pub new_incident_name: String,
    pub new_incident_description: String,
    pub new_incident_x_coordenate: String,
    pub new_incident_y_coordenate: String,
    
    pub new_drone_id: String,
    pub new_drone_password: String,
    pub new_drone_anchor_x_coordenate: String,
    pub new_drone_anchor_y_coordenate: String,
    
    pub ui_receiver: mpsc::Receiver<String>,
    current_layout: Layout,
    tiles: Tiles,
    map_memory: MapMemory,
}

impl UIApplication {
    pub fn new(egui_ctx: Context) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            client: Arc::new(Mutex::new(Client::new(sender))),
            new_incident_name: "".to_owned(),
            new_incident_description: "".to_owned(),
            new_incident_x_coordenate: "".to_owned(),
            new_incident_y_coordenate: "".to_owned(),
            new_drone_id: "".to_owned(),
            new_drone_password: "".to_owned(),
            new_drone_anchor_x_coordenate: "".to_owned(),
            new_drone_anchor_y_coordenate: "".to_owned(),
            ui_receiver: receiver,
            current_layout: Layout::IncidentMap,
            tiles: Tiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),
        }
    }
}

impl eframe::App for UIApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut client = self.client.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.image(egui::include_image!("images/logo.png"));
                ui.heading(egui::RichText::new("Monitor").size(30.0));
            });
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    match client.client_run() {
                        Ok(_) => println!("Conectado"),
                        Err(e) => {
                            println!("Error al conectar: {:?}", e);
                        }
                    }
                }
                ui.add_space(500.0);
                

                if *client.connection_status.lock().unwrap() == "connected" {
                    ui.label(egui::RichText::new("Connected").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Disconnected").color(egui::Color32::RED));
                }
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.current_layout, 
                    Layout::IncidentMap,
                    "Map");
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
                ui.selectable_value(
                    &mut self.current_layout,
                    Layout::DroneOperations,
                    "Drones",
                );
            });

            ui.add_space(20.0);

            if self.current_layout == Layout::IncidentMap {
                let center_position = Position::from_lon_lat(-58.3717, -34.6081);

                let incident_list = client.incident_list.lock().unwrap();
                let drone_list = client.drone_list.lock().unwrap();
        
                let places_plugin = update_places(incident_list, drone_list);
                
                ui.add(
                    Map::new(Some(&mut self.tiles), &mut self.map_memory, center_position)
                        .with_plugin(places_plugin),
                );
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
                        match client.new_incident(
                            &self.new_incident_name,
                            &self.new_incident_description,
                            &self.new_incident_x_coordenate,
                            &self.new_incident_y_coordenate,
                        ) {
                            Ok(_) => println!("Nuevo incidente enviado"),
                            Err(e) => {
                                println!("Error al publicar mensaje: {:?}", e);
                            }
                        }
                    }
                });
            }

            if self.current_layout == Layout::IncidentList {
                let incidents = client.incident_list.lock().unwrap();
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
                                    ui.label("x,y");
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
                        ui.add(egui::TextEdit::singleline(&mut self.new_drone_id).desired_width(340.0));
                    });
            
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add_space(20.0);
                        ui.add(egui::TextEdit::singleline(&mut self.new_drone_password).desired_width(340.0));
                    });
            
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Anchor Point Coordinates:");

                        ui.add_space(10.0);
                        ui.label("x:");
                        ui.add(egui::TextEdit::singleline(&mut self.new_drone_anchor_x_coordenate).desired_width(100.0));

                        ui.add_space(10.0);
                        ui.label("y:");
                        ui.add(egui::TextEdit::singleline(&mut self.new_drone_anchor_y_coordenate).desired_width(100.0));
                        ui.add_space(263.0);
                        if ui.button("Register").clicked() {
                            match client.new_drone(
                                &self.new_drone_id,
                                &self.new_drone_password,
                                &self.new_drone_anchor_x_coordenate,
                                &self.new_drone_anchor_y_coordenate,
                            ) {
                                Ok(_) => println!("New drone created"),
                                Err(e) => {
                                    println!("Error creating drone: {:?}", e);
                                }
                            }
                        }
                    });
                });

                ui.add_space(20.0);

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label("Active Drones:");

                    ui.add_space(10.0);
                    let drones = client.drone_list.lock().unwrap();

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
                                        let position = format!("({}, {})", drone.x_coordinate, drone.y_coordinate);
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

fn update_places(incidents: MutexGuard<Vec<Incident>>, drones: MutexGuard<Vec<Drone>>) -> Places{
    let mut places = vec![];

    let incident_data: Vec<_> = incidents.iter().collect();
    let drone_data: Vec<_> = drones.iter().collect();

    for incident in incident_data {
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

    for drone in drone_data {
        let color = match drone.state.as_str() {
            "Free" => Color32::from_rgb(0, 255, 0), // Green
            "Attending Incident" => Color32::from_rgb(255, 0, 0), // Red
            "Travelling" => Color32::from_rgb(255, 255, 0), // Yellow
            _ => Color32::from_rgb(255, 255, 255), // White
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

    Places::new(places)

}