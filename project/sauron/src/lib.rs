use model::{
    connect_package::ConnectPackage,
    qos::QoS,
    encoded_string::EncodedString,
    constants::*
};

mod errors;
mod model;

#[allow(clippy::manual_map)] pub fn connect(
    client_id: String,
    clean_session: bool,
    keep_alive: u16,
    will: Option<(QoS, String, String)>, // TODO no se va QoS o u8
    user: Option<(String, Option<String>)>,
) -> ConnectPackage {
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

    let (will_flag, will_qos, will_retain) = match will {
        Some((qos, _, _)) => (true, qos, false),
        None => (false, QoS::AtMost, false),
    };

    let (username_flag, password_flag) = match &user {
        Some((_, password)) => (true, password.is_some()),
        None => (false, false),
    };

    let variable_header_length = PACKAGE_IDENTIFIER_LENGTH + CONNECT_LENGTH;
    let mut payload_lenght = client_id.get_length();

    if let Some((will_topic, will_message)) = &payload_will {
        payload_lenght += will_topic.get_length();
        payload_lenght += will_message.get_length();
    }

    if let Some((username, password)) = &payload_user {
        payload_lenght += username.get_length();

        if let Some(password) = password {
            payload_lenght += password.get_length();
        }
    }

    let remaining_length = variable_header_length + payload_lenght;

    ConnectPackage::new(
        remaining_length,
        0,
        username_flag,
        password_flag,
        will_retain,
        will_qos,
        will_flag,
        clean_session,
        keep_alive,
        client_id,
        payload_will,
        payload_user, 
    )
}
