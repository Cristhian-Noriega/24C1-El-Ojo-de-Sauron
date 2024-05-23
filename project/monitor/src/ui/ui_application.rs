use gtk::prelude::*;
use gtk::{Builder, Button, Label, Window};

pub struct UIApplication {
    builder: Builder,
}

impl Default for UIApplication {
    fn default() -> Self {
        if let Err(err) = gtk::init() {
            eprintln!("Error initializing GTK: {}", err);
        }
        let builder = Builder::from_file("monitor/src/ui/view.glade");
        UIApplication { builder }
    }
}

impl UIApplication {
    pub fn start() {
        let mut window = UIApplication::default();
        window.show_window();
    }

    pub fn show_window(&mut self) {
        let window: Option<Window> = self.builder.get_object("window");

        println!("{:?}", window);

        if let Some(w) = window {
            self.connect_button();
            w.show_all();
        };
        gtk::main();
    }

    fn connect_button(&mut self) {
        let button_connect: Option<Button> = self.builder.get_object("button_connect");
        let builder = self.builder.clone();

        println!("{:?}", button_connect);

        if let Some(b) = button_connect {
            b.connect_clicked(move |_| {
                // Aca deberia realizar el connect y obtener la respuesta
                Self::set_label(&builder, "label_response", "Connected!");
            });
        }
    }

    fn set_label(builder: &Builder, key: &str, value: &str) {
        let label: Option<Label> = builder.get_object(key);
        if let Some(l) = label {
            l.set_text(value);
        }
    }
    
}