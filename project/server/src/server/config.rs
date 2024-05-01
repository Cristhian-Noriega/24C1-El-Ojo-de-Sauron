//Configuración: debe incluir todos los parámetros necesarios para la ejecución del servidor, como el puerto, direccion IP, etc. 
//(no esta permitido definir estos valores mediante constantes en el código)

pub struct Config {
    port: u16,
    address: String,
    log_file: String
}

impl Config{
    pub fn new(config_file: &str) -> Option<Config> {
        let mut path = PathBuf::new(config_file);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let map = Self::parse_config(reader)?;

        let port = Self::get_port(&map)?;
        let address = Self::get_address(&map)?;
        let log_file = Self::get_log_file(&map)?;

        Some(Self { port, address, log_file })
    }

    fn parse_config<R: BufRead>(reader: R) -> Option<HashMap<String, String>> {
        let mut map = HashMap::new();
        for line in reader.lines() {
            let line = line.ok()?;
            let mut parts = line.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Some(map)
    }

    fn get_port(map: &HashMap<String, String>) -> Option<u16> {
        map.get("port")?.parse().ok()
    }

    fn get_address(map: &HashMap<String, String>) -> Option<String> {
        map.get("address").cloned()
    }

    fn get_log_file(map: &HashMap<String, String>) -> Option<String> {
        map.get("log_file").cloned()
    }
}