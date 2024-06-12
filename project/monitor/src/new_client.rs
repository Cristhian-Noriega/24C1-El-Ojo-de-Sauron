use std::{
    io::{ErrorKind, Write},
    net::TcpStream,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use mqtt::model::{
    components::encoded_string::EncodedString,
    packet::Packet,
    packets::{connect::Connect, puback::Puback},
};

use crate::{
    channels_tasks::{MonitorAction, UIAction},
    monitor::Monitor,
    ui_application::UIApplication,
};

pub fn client_run(address: String) -> Result<(), String> {
    let (monitor_sender, from_monitor_receiver) = channel();
    let (ui_sender, from_ui_receiver) = channel();
    let (internal_monitor_sender, internal_monitor_receiver) = channel();

    let stream = match connect_to_server(address) {
        Ok(stream) => stream,
        Err(e) => {
            return Err(format!("Error connecting to server: {:?}", e));
        }
    };

    let stream = Arc::new(Mutex::new(stream));

    let monitor = Monitor::new();
    let monitor = Arc::new(Mutex::new(monitor));

    let cloned_monitor = monitor.clone();
    let cloned_stream = stream.clone();

    let monitor_stream_listener_thread = std::thread::spawn(move || {
        monitor_stream_listener(
            cloned_monitor,
            cloned_stream,
            monitor_sender,
            internal_monitor_sender,
        );
    });

    let cloned_monitor = monitor.clone();
    let cloned_stream = stream.clone();

    let monitor_stream_writer_thread = std::thread::spawn(move || {
        monitor_stream_writer(
            cloned_monitor,
            cloned_stream,
            from_ui_receiver,
            internal_monitor_receiver,
        );
    });

    match start_ui(ui_sender, from_monitor_receiver) {
        Ok(_) => {}
        Err(_) => {
            return Err("Error starting UI".to_string());
        }
    }

    monitor_stream_listener_thread.join().unwrap();
    monitor_stream_writer_thread.join().unwrap();

    Ok(())
}

fn connect_to_server(address: String) -> std::io::Result<TcpStream> {
    println!("\nConnecting to address: {:?}", address);
    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = b"drone".to_vec();
    let client_id = EncodedString::new(client_id_bytes);
    let will = None;
    let login = None; // TODO: Add login
    let connect = Connect::new(false, 0, client_id, will, login);

    let _ = to_server_stream.write(connect.to_bytes().as_slice());

    match Packet::from_bytes(&mut to_server_stream) {
        Ok(Packet::Connack(connack)) => {
            println!(
                "Received Connack packet with return code: {:?} and sessionPresent: {:?}\n",
                connack.connect_return_code(),
                connack.session_present()
            );
            Ok(to_server_stream)
        }
        _ => Err(std::io::Error::new(ErrorKind::Other, "No connack recibed")),
    }
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

fn monitor_stream_listener(
    monitor: Arc<Mutex<Monitor>>,
    stream: Arc<Mutex<TcpStream>>,
    monitor_sender: Sender<MonitorAction>,
    internal_monitor_sender: Sender<Puback>,
) {
    loop {
        let locked_stream = stream.lock().unwrap();
        let mut stream = locked_stream.try_clone().unwrap();

        match Packet::from_bytes(&mut stream) {
            Ok(Packet::Puback(puback)) => {
                internal_monitor_sender.send(puback).unwrap();
            }
            Ok(Packet::Publish(publish)) => {
                
                // DRONE DATA

                let topics

                // CAMERA DATA
                // ATTENDING INCIDENT

            }
            _ => {}
        }
    }
}

fn monitor_stream_writer(
    monitor: Arc<Mutex<Monitor>>,
    stream: Arc<Mutex<TcpStream>>,
    from_ui_receiver: Receiver<UIAction>,
    internal_monitor_receiver: Receiver<Puback>,
) {
    loop {
        match from_ui_receiver.recv() {
            // Ok(UIAction::Connect) => {
            //     // lógica de conexión
            // }
            // Ok(UIAction::Disconnect) => {
            //     // lógica de desconexión
            // }
            Ok(UIAction::RegistrateDrone(drone_registration)) => {
                // lógica de registro de dron
            }
            Ok(UIAction::RegistrateIncident(incident_registration)) => {
                // lógica de registro de incidente
            }
            _ => {}
        }

        match internal_monitor_receiver.recv() {
            Ok(puback) => {
                // lógica de puback
            }
            _ => {}
        }
    }
}
