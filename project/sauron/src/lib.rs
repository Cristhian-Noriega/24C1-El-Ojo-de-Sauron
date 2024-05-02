use model::{encoded_string::EncodedString, packet::Packet, packets::connect_packet::ConnectPacket, qos::QoS};

mod errors;
mod model;

#[allow(clippy::manual_map)]
pub fn connect(
    client_id: String,
    packet_identifier: u16,
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

    Packet::Connect(ConnectPacket::new(
        packet_identifier,
        clean_session,
        keep_alive,
        client_id,
        will,
        user,
    ))
}
