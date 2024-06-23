use monitor::monitor::Monitor;
use common::incident::{Incident, IncidentStatus};
use drone::drone::Drone;
use drone::drone_status::{DroneStatus, TravelLocation};
use camera_system::camera_system::CameraSystem;
use camera_system::camera::Camera;

#[test]
fn test_new_incident() {
    let mut monitor = Monitor::new();
    let incident = Incident::new(
        "incident1".to_string(),
        "incident1".to_string(),
        "incident1".to_string(),
        2.0,
        2.0,
        IncidentStatus::Pending,
    );

    // Monitor
    monitor.new_incident(incident.clone());
    assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

    // Drone
    let mut drone = Drone::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 5.0);
    assert_eq!(drone.data(), "1;1;0;100");
    drone.set_incident(Some(incident.clone()));
    assert_eq!(drone.incident().unwrap(), incident);
    drone.set_status(DroneStatus::Travelling(TravelLocation::Incident));
    assert_eq!(drone.status(), DroneStatus::Travelling(TravelLocation::Incident));
    drone.travel_to(2.0, 2.0);
    assert_eq!(drone.data(), "1.7071067811865475;1.7071067811865475;4;99");

    // Camara
    let mut camera_system = CameraSystem::new();
    let camera = Camera::new(
        1 as u8,
        1.5,
        1.5,
        3.0,
    );
    camera_system.add_camera(camera);
    let camera_data1 = camera_system.cameras_data();
    camera_system.new_incident(incident.clone());
    let camera_data2 = camera_system.cameras_data();
    assert_eq!(camera_data1, "1;1.5;1.5;0");
    assert_eq!(camera_data2, "1;1.5;1.5;1");
}  

#[test]
fn test_attending_incident_once() {
    let mut monitor = Monitor::new();
    let incident = Incident::new(
        "incident1".to_string(),
        "incident1".to_string(),
        "incident1".to_string(),
        1.0,
        1.0,
        IncidentStatus::Pending,
    );

    monitor.new_incident(incident.clone());
    assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

    let incident = monitor.attend_incident(incident.uuid).unwrap();

    assert_eq!(incident.status, IncidentStatus::Pending);
}

#[test]
fn test_attending_incident_twice() {
    let mut monitor = Monitor::new();
    let incident = Incident::new(
        "incident1".to_string(),
        "incident1".to_string(),
        "incident1".to_string(),
        1.0,
        1.0,
        IncidentStatus::Pending,
    );


    monitor.new_incident(incident.clone());
    assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

    let incident = monitor.attend_incident(incident.uuid).unwrap();
    let incident = monitor.attend_incident(incident.uuid).unwrap();

    assert_eq!(incident.status, IncidentStatus::InProgress);
}

#[test]
fn test_resolvable_incident() {
    let mut monitor = Monitor::new();
    let incident = Incident::new(
        "incident1".to_string(),
        "incident1".to_string(),
        "incident1".to_string(),
        1.0,
        1.0,
        IncidentStatus::Pending,
    );

    monitor.new_incident(incident.clone());
    assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

    monitor.set_resolvable_incident(incident.uuid.clone());
    let incident = monitor.get_incident(&incident.uuid).unwrap();
    assert_eq!(incident.status, IncidentStatus::Resolvable);
}

#[test]
fn test_solve_incident() {
    let mut monitor = Monitor::new();
    let incident = Incident::new(
        "incident1".to_string(),
        "incident1".to_string(),
        "incident1".to_string(),
        1.0,
        1.0,
        IncidentStatus::Pending,
    );


    monitor.new_incident(incident.clone());
    assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

    monitor.set_resolved_incident(incident.uuid.clone());
    let incident = monitor.get_incident(&incident.uuid).unwrap();
    assert_eq!(incident.status, IncidentStatus::Resolved);
}

#[test]
fn test_edit_incident() {
    let mut monitor = Monitor::new();
    let incident = Incident::new(
        "incident1".to_string(),
        "incident1".to_string(),
        "incident1".to_string(),
        1.0,
        1.0,
        IncidentStatus::Pending,
    );

    monitor.new_incident(incident.clone());
    assert_eq!(monitor.get_incident(&incident.uuid).unwrap(), &incident);

    let incident = monitor
        .edit_incident(incident.uuid, "incident1 edit".to_string(), "incident1 edit".to_string())
        .unwrap();
    assert_eq!(incident.name, "incident1 edit");
    assert_eq!(incident.description, "incident1 edit");
}