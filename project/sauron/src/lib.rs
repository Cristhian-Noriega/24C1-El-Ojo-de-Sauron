use model::{
    encoded_string::EncodedString, fixed_header::FixedHeader, packet::Packet,
    packets::connack::Connack, packets::connect::Connect, packets::publish::Publish,
    packets::suback::Suback, packets::subscribe::Subscribe, packets::puback::Puback,
    packets::disconnect::Disconnect, packets::pingreq::Pingreq, packets::pingresp::Pingresp,
    packets::unsubscribe::Unsubscribe, qos::QoS, return_codes::connack_return_code::ConnackReturnCode,
};

mod errors;
mod model;

#[allow(clippy::manual_map)]
pub fn connect(
    client_id: String,
    clean_session: bool,
    keep_alive: u16,
    will: Option<(QoS, bool, String, String)>, // Tendría una estructura para esto
    user: Option<(String, Option<String>)>,    // quiza para esto tmb
) -> Packet {
    let client_id = EncodedString::from_string(&client_id);

    let will = match will {
        Some((qos, retain, topic, message)) => Some((
            qos,
            retain,
            EncodedString::from_string(&topic),
            EncodedString::from_string(&message),
        )),
        None => None,
    };

    let user = match &user {
        Some((username, password)) => Some((
            EncodedString::from_string(username),
            match password {
                Some(password) => Some(EncodedString::from_string(password)),
                None => None,
            },
        )),
        None => None,
    };

    Packet::Connect(Connect::new(
        clean_session,
        keep_alive,
        client_id,
        will,
        user,
    ))
}

pub fn connack(session_present: bool, return_code: ConnackReturnCode) -> Packet {
    Packet::Connack(Connack::new(session_present, return_code))
}

pub fn publish(
    dup: bool,
    qos: QoS,
    retain: bool,
    topic_name: String,
    package_identifier: Option<u16>,
    payload: Vec<u8>,
) -> Packet {
    let topic_name = EncodedString::from_string(&topic_name);

    Packet::Publish(Publish::new(
        dup,
        qos,
        retain,
        topic_name,
        package_identifier,
        payload,
    ))
}
