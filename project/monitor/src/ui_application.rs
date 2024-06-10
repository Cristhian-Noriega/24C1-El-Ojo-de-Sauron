use crate::{client::Client, monitor::Monitor};
use eframe::egui;
use egui::Context;
use egui_extras::{Column, TableBuilder};
use std::sync::{mpsc, Arc, Mutex};
use walkers::{sources::OpenStreetMap, Map, MapMemory, Position, Tiles};

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
    monitor: Monitor,
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
            monitor: Monitor::new(),
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
                    match client.client_run(&self.monitor) {
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
                ui.add(Map::new(
                    Some(&mut self.tiles),
                    &mut self.map_memory,
                    Position::from_lon_lat(-58.3717, -34.6081),
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
                        match self.monitor.new_incident(
                            &self.new_incident_name,
                            &self.new_incident_description,
                            &self.new_incident_x_coordenate,
                            &self.new_incident_y_coordenate,
                            &client,
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
                            match self.monitor.new_drone(
                                &self.new_drone_id,
                                &self.new_drone_password,
                                &self.new_drone_anchor_x_coordenate,
                                &self.new_drone_anchor_y_coordenate,
                                &client,
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
