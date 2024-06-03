use std::sync::{mpsc, Arc, Mutex};
use eframe::egui;
use egui_extras::{TableBuilder, Column};
use crate::client::Client;

#[derive(PartialEq)]
enum Layout { IncidentMap, IncidentList, NewIncident }

pub struct UIApplication {
    client: Arc<Mutex<Client>>,
    pub new_incident_name: String,
    pub new_incident_description: String,
    pub new_incident_x_coordenate: String,
    pub new_incident_y_coordenate: String,
    pub ui_receiver: mpsc::Receiver<String>,
    current_layout: Layout,
}

impl Default for UIApplication {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            client: Arc::new(Mutex::new(Client::new(sender))),
            new_incident_name: "".to_owned(),
            new_incident_description: "".to_owned(),
            new_incident_x_coordenate: "".to_owned(),
            new_incident_y_coordenate: "".to_owned(),
            ui_receiver: receiver,
            current_layout: Layout::NewIncident,
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

                if *client.connection_status.lock().unwrap() == "connected"{
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
            });

            ui.add_space(20.0);


            if self.current_layout == Layout::IncidentMap {
                ui.label("Map of incidents");
            }

            if self.current_layout == Layout::NewIncident{
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
                        match client.new_incident(&self.new_incident_name, &self.new_incident_description, &self.new_incident_x_coordenate, &self.new_incident_y_coordenate) {
                            Ok(_) => println!("Nuevo incidente enviado"),
                            Err(e) => {
                                println!("Error al publicar mensaje: {:?}", e);
                            }
                        }
                    }
                });
            }

            if self.current_layout == Layout::IncidentList{
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
                        body.row(50.0, |mut row| {
                            row.col(|ui| {
                                ui.label("550e8400-e29b-41d4-a716-446655440000");
                            });
                            row.col(|ui| {
                                ui.label("Robbery");
                            });
                            row.col(|ui| {
                                ui.label("Test description");
                            });
                            row.col(|ui| {
                                ui.label("35.0, 45.0");
                            });
                            row.col(|ui| {
                                ui.label("Active");
                            });
                        });
                    });
            }
        });
    }
}