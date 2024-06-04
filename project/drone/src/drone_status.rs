#[derive(Debug, Clone)]
pub enum DroneStatus {
    Free,
    Traveling,
    Busy,
    Charging,
}

impl std::fmt::Display for DroneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DroneStatus::Free => write!(f, "0"),
            DroneStatus::Traveling => write!(f, "1"),
            DroneStatus::Busy => write!(f, "2"),
            DroneStatus::Charging => write!(f, "3"),
        }
    }
}
