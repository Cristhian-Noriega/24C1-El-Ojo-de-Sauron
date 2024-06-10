#[derive(Debug, Clone, PartialEq)]
pub enum DroneStatus {
    Free,
    Traveling,
    AttendingIncident,
    Charging,
}

impl std::fmt::Display for DroneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DroneStatus::Free => write!(f, "0"),
            DroneStatus::Traveling => write!(f, "1"),
            DroneStatus::AttendingIncident => write!(f, "2"),
            DroneStatus::Charging => write!(f, "3"),
        }
    }
}
