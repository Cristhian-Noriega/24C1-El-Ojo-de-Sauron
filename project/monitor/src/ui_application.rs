use crate::client::Client;
use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};

pub struct UIApplication {
    client: Arc<Mutex<Client>>,
    pub topic: String,
    pub message: String,
    pub ui_receiver: mpsc::Receiver<String>,
}

impl Default for UIApplication {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            client: Arc::new(Mutex::new(Client::new(sender))),
            topic: "".to_owned(),
            message: "".to_owned(),
            ui_receiver: receiver,
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
                ui.label("Topic:");
                ui.add_space(18.0);
                ui.text_edit_singleline(&mut self.topic);
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Message:");
                ui.text_edit_multiline(&mut self.message);
            });
            if ui.button("Publish").clicked() {
                match client.publish(&self.topic, &self.message) {
                    Ok(_) => println!("Mensaje publicado"),
                    Err(e) => {
                        println!("Error al publicar mensaje: {:?}", e);
                    }
                }
            }

            ui.add_space(20.0);

            ui.heading(egui::RichText::new("Last message received").size(20.0));
            if let Ok(new_message) = self.ui_receiver.try_recv() {
                *client.response_text.lock().unwrap() = new_message;
            }

            let response_text = format!("{:?}", *client.response_text.lock().unwrap());
            if ui.label(response_text).clicked() {
                ui.ctx().request_repaint();
            }
        });
    }
}
