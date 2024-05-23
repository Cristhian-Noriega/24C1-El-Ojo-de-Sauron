use crate::ui::ui_application::UIApplication;

pub mod ui;

fn main() -> Result<(), eframe::Error>{
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "Monitor",
        options,
        Box::new(|_cc| {
            Box::<UIApplication>::default()
        }),
    )
}