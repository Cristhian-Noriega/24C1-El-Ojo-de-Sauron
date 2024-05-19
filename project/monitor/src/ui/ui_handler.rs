use super::ui_application::UIApplication;

pub struct UIHandler {}

impl UIHandler {
    // pub fn start() -> JoinHandle<()> {
    //     thread::spawn(move || {
    //         if let Err(err) = gtk::init() {
    //             eprintln!("Error initializing GTK: {}", err);
    //         }

    //         UIApplication::start();
    //     })
    // }

    pub fn start() {
        UIApplication::start();
    }
}