use eframe::egui;

pub struct UIApplication {
    response: String,
}

impl Default for UIApplication {
    fn default() -> Self {
        Self {
            response: "no response".to_owned(),
        }
    }
}

impl eframe::App for UIApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monitor");
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    self.response = "Connected!".to_owned();
                }
                ui.vertical_centered(|ui| {
                    ui.label("Response");
                    ui.label(format!("{}", self.response));
                });
            });
        });
    }
}