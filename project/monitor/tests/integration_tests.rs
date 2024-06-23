use monitor::monitor::Monitor;
use common::incident::{Incident, IncidentStatus};

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