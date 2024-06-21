//! The drone system is a program that simulates a single drone. It recieves messages from the monitor and
//! reacts to them depending on the situation.

use std::path::Path;
use config::Config;
use std::env::args;

mod client;
mod drone;
mod drone_status;
mod config;

static CLIENT_ARGS: usize = 2;

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválidos");
        let app_name = &argv[0];
        println!("{:?} <config-path>", app_name);

        return;
    }

    let path = Path::new(&argv[1]);

    let config = match Config::from_file(path) {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading the configuration file: {:?}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = client::client_run(config) {
        println!("Error: {:?}", e);
    }
}

// Dron
//      ID (xxx) (int)
//      Coordenadas actuales
//      Ubicación predeterminada
//      Área máxima (interno, radio)
//      Batería
//      Ubicación central (de carga)
//      Nivel mínimo de batería
//      Estado (Libre, Viajando, Ocupado, Cargando)

// Incializar el drone con archivo de config y conectarse al servidor con usuario y contraseña esperar el PUBACK
// Suscribirse a new-incident

// Crear un thread que actualize cada X segundos el estado del drone
//      Si la bateria es menor al mínimo -> Volver a la ubicación central

// Cuando llega un publish ->
//      Si es new-incident ->
//          Si le interesa (batería, área máxima)
//              Mueve al drone a la zona
//              Suscribirse a attending-incident/uuid

// Si llega a la zona ->
//     Publica en attending-incident/uuid
//     Si la cuenta = 1 ->
//         Suscribirse a close-incident/uuid

//      Si es close-incident/uuid ->
//          Mueve al drone a la ubi predeterminada
//          Desuscribirse de close-incident/uuid

// Si recibe un attending-incident/uuid ->
//      Si ya había llegado a la zona ->
//          Suscribirse a close-incident/uuid
//          Desuscribirse de attending-incident/uuid
//      Si no llego y la cuenta = 0
//          Suma 1 a la cuenta
//      Si no llego y la cuenta = 1
//          Desuscribirse de attending-incident/uuid
//          Vuelve a la ubi predeterminada
