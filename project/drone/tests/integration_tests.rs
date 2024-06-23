use drone::drone::Drone;
use drone::drone_status::{DroneStatus, TravelLocation};

#[test]
fn test_drone_data() {
    let drone = Drone::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    assert_eq!(drone.data(), "1;1;0;100");
}

#[test]
fn test_travelling_to_central_drone_data() {
    let mut drone = Drone::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);

    drone.set_status(DroneStatus::Travelling(TravelLocation::Central));
    assert_eq!(drone.data(), "1;1;2;100");
}

#[test]
fn test_attending_incident_drone_data() {
    let mut drone = Drone::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);

    drone.set_status(DroneStatus::AttendingIncident);
    assert_eq!(drone.data(), "1;1;1;100");
}

#[test]
fn test_drone_travel_to(){
    let mut drone = Drone::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    drone.travel_to(3.0, 3.0);
    assert_eq!(drone.data(), "1.7071067811865475;1.7071067811865475;0;99");
}