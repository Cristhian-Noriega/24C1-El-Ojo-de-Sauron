use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Box, Orientation};

const DEFAULT_WINDOW_WIDTH: i32 = 1200;
const DEFAULT_WINDOW_HEIGHT: i32 = 700;

pub struct UIApplication {}

impl UIApplication {
    pub fn start() {
        let app = Application::builder()
            .application_id("com.fiuba.monitor")
            .build();
        app.connect_activate(build_home_window);
        app.run();
    }
    
}

fn build_home_window(app: &Application){
    // Configuro ventana del homepage
    let home_window = ApplicationWindow::new(app);

    home_window.set_title("Monitor");

    home_window.set_default_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

    let home_box = Box::new(Orientation::Vertical, 0);
    let title_label = gtk::Label::new(None);
    let map_button = Button::with_label("Map");
    let incident_button = Button::with_label("Incidents");

    title_label.set_markup("<span size='xx-large'>Monitor</span>");
    map_button.set_size_request(200, 50);
    incident_button.set_size_request(200, 50);

    home_box.pack_start(&title_label, false, false, 10);
    home_box.pack_start(&map_button, false, false, 20);
    home_box.pack_start(&incident_button, false, false, 20);

    home_window.set_child(Some(&home_box));

    home_window.show_all();
}

// fn build_map_window(app: &Application){
//     let window = ApplicationWindow::new(app);
//     window.set_title("Map");
//     window.set_default_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

//     let map_box = Box::new(Orientation::Vertical, 0);
//     let home_button = Button::with_label("Home");
//     let image = Image::from_file("monitor/src/ui/images/map.png");

//     map_box.pack_start(&home_button, false, false, 10);
//     map_box.pack_start(&image, false, false, 20);

//     window.set_child(Some(&map_box));   

//     window.show_all(); 
// }