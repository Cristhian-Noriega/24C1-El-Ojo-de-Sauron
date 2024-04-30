use model::{
    package::Package,
    package_components::{fixed_header_components::qos::QoS, payload_components},
};

mod errors;
mod model;

pub fn send_connect(client_id: Vec<u8>) {
    let variable_header_content =
        model::package_components::variable_header_components::contents::connect_variable_header_content::ConnectVariableHeaderContent::new(
            false,
            false,
            false,
            QoS::AtLeastOnce,
            false,
            false,
            0,
            0,
        );
    let payload =
        payload_components::connect_payload::ConnectPayload::new(client_id, None, None, None, None);

    let package = Package::build_connect(client_id, variable_header_content, payload).unwrap();
    let package_bytes = package.into_bytes();

    //send
}
