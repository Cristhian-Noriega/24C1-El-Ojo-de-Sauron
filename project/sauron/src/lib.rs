use model::{
    package::Package,
    package_components::{
        fixed_header::FixedHeader,
        fixed_header_components::{
            control_packet_type::ControlPacketType, fixed_header_flags::FixedHeaderFlags, qos::QoS,
        },
        payload::Payload,
        payload_components::{
            contents::payload_content_connect::PayloadContentConnect, encoded_string::EncodedString,
        },
        variable_header::VariableHeader,
        variable_header_components::{
            contents::variable_header_content_connect::VariableHeaderContentConnect,
            variable_header_content::VariableHeaderContent,
        },
    },
};

mod errors;
mod model;

pub fn connect(
    client_id: String,
    clean_session: bool,
    keep_alive: u16,
    will: Option<(QoS, String, String)>, // TODO no se va QoS o u8
    user: Option<(String, Option<String>)>,
) -> Package {
    let client_id = EncodedString::from_string(&client_id);

    let payload_will: Option<(EncodedString, EncodedString)> = match &will {
        Some((_, topic, message)) => Some((
            EncodedString::from_string(topic),
            EncodedString::from_string(message),
        )),
        None => None,
    };

    let payload_user = match &user {
        Some((username, password)) => Some((
            EncodedString::from_string(username),
            match password {
                Some(password) => Some(EncodedString::from_string(password)),
                None => None,
            },
        )),
        None => None,
    };

    let payload = Payload::Connect(PayloadContentConnect::new(
        client_id,
        payload_will,
        payload_user,
    ));

    let (will_flag, will_qos, will_retain_flag) = match will {
        Some((qos, _, _)) => (true, qos, false),
        None => (false, QoS::AtMostOnce, false),
    };

    let (username_flag, password_flag) = match &user {
        Some((_, password)) => (true, password.is_some()),
        None => (false, false),
    };

    let variable_header = VariableHeader::new(
        0,
        VariableHeaderContent::Connect(VariableHeaderContentConnect::new(
            clean_session,
            will_flag,
            will_qos,
            will_retain_flag,
            username_flag,
            password_flag,
            keep_alive,
        )),
    );

    let remaining_length = variable_header.get_length() + payload.get_length();

    let fixed_header = FixedHeader::new(
        ControlPacketType::Connect,
        FixedHeaderFlags::Reserved,
        remaining_length,
    );

    Package::new(fixed_header, Some(variable_header), Some(payload))
}
