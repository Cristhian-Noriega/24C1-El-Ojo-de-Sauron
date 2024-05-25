use eframe::egui;
use crate::client::Client;


pub struct UIApplication {
    client: Client
}

impl Default for UIApplication {
    fn default() -> Self {
        Self {
            client: Client::new()
        }
    }
}

impl eframe::App for UIApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.image(egui::include_image!("images/logo.png"));
                ui.heading(egui::RichText::new("Monitor").size(30.0));
            });
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    self.client.send_connect();
                }
                ui.add_space(500.0);

                if self.client.connection_status == "connected" {
                    ui.label(egui::RichText::new("Connected").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Disconnected").color(egui::Color32::RED));
                }
            });

            ui.add_space(20.0);

            ui.heading(egui::RichText::new("Last message received").size(20.0));
            ui.horizontal_centered(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Bytes").size(15.0));
                    ui.label(format!("{}", self.client.response_bytes));
                });
                ui.add_space(100.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Text").size(15.0));
                    ui.label(format!("{}", self.client.response_text));
                });
            });
        });
    }
}