/// Represents the status of a drone
#[derive(Debug, Clone, PartialEq)]
pub enum DroneStatus {
    Free,
    Travelling(TravelLocation),
    AttendingIncident,
    Recharging,
    Interrupted,
}

/// Represents the location of the drone when travelling
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TravelLocation {
    Central,
    Anchor,
    Incident,
}

impl std::fmt::Display for DroneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DroneStatus::Free => write!(f, "0"),
            DroneStatus::AttendingIncident => write!(f, "1"),
            DroneStatus::Travelling(TravelLocation::Central) => write!(f, "2"),
            DroneStatus::Travelling(TravelLocation::Anchor) => write!(f, "3"),
            DroneStatus::Travelling(TravelLocation::Incident) => write!(f, "4"),
            DroneStatus::Recharging => write!(f, "5"),
            DroneStatus::Interrupted => write!(f, "6"),
        }
    }
}

impl DroneStatus {
    pub fn to_str(&self) -> &str {
        match self {
            DroneStatus::Free => "Free",
            DroneStatus::AttendingIncident => "Attending Incident",
            DroneStatus::Travelling(TravelLocation::Central) => "Travelling to Central",
            DroneStatus::Travelling(TravelLocation::Anchor) => "Travelling to Anchor",
            DroneStatus::Travelling(TravelLocation::Incident) => "Travelling to Incident",
            DroneStatus::Recharging => "Charging",
            DroneStatus::Interrupted => "Interrupted",
        }
    }
    pub fn get_status_from_str(string: &str) -> Self {
        match string {
            "0" => DroneStatus::Free,
            "1" => DroneStatus::AttendingIncident,
            "2" => DroneStatus::Travelling(TravelLocation::Central),
            "3" => DroneStatus::Travelling(TravelLocation::Anchor),
            "4" => DroneStatus::Travelling(TravelLocation::Incident),
            "5" => DroneStatus::Recharging,
            "6" => DroneStatus::Interrupted,
            _ => DroneStatus::Free,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drone_status_display() {
        assert_eq!(format!("{}", DroneStatus::Free), "0");
        assert_eq!(format!("{}", DroneStatus::AttendingIncident), "1");
        assert_eq!(
            format!("{}", DroneStatus::Travelling(TravelLocation::Central)),
            "2"
        );
        assert_eq!(
            format!("{}", DroneStatus::Travelling(TravelLocation::Anchor)),
            "3"
        );
        assert_eq!(
            format!("{}", DroneStatus::Travelling(TravelLocation::Incident)),
            "4"
        );
        assert_eq!(format!("{}", DroneStatus::Recharging), "5");
    }
}
