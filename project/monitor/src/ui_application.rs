use eframe::egui;
use crate::client::Client;


pub struct UIApplication {
    response: String,
    client: Client
}

impl Default for UIApplication {
    fn default() -> Self {
        Self {
            response: "no response".to_owned(),
            client: Client::new()
        }
    }
}

impl eframe::App for UIApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monitor");
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    self.client.send_connect();
                    self.response = self.client.response.clone();
                }
                ui.vertical_centered(|ui| {
                    ui.label("Response");
                    ui.label(format!("{}", self.response));
                });
            });
        });
    }
}