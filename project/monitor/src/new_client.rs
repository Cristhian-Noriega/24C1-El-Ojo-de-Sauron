use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    channels_tasks::{MonitorAction, UIAction},
    monitor::Monitor,
    ui_application::UIApplication,
};

pub fn client_run(address: String) -> Result<(), String> {
    let (monitor_sender, from_monitor_receiver) = channel();
    let (ui_sender, from_ui_receiver) = channel();

    let ui_thread = std::thread::spawn(move || {
        start_ui(ui_sender, from_monitor_receiver);
    });

    let monitor_thread = std::thread::spawn(move || {
        start_monitor(address, monitor_sender, from_ui_receiver);
    });

    ui_thread.join().unwrap();
    monitor_thread.join().unwrap();

    Ok(())
}

fn start_ui(
    ui_sender: Sender<UIAction>,
    from_monitor_receiver: Receiver<MonitorAction>,
) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Monitor",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(UIApplication::new(
                cc.egui_ctx.clone(),
                ui_sender,
                from_monitor_receiver,
            ))
        }),
    )
}

fn start_monitor(
    address: String,
    monitor_sender: Sender<MonitorAction>,
    ui_receiver: Receiver<UIAction>,
) {
}
