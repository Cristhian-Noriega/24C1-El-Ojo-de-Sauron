

enum Payload {
    ConnectPayload,
    PublshPayload,
    SubscribePayload,
    SubAckPayload,
    UnsubscribePayload,
    NoPayload,
}
//The Server MUST allow ClientIds which are between 1 and 23 UTF-8 encoded bytes in length, and that contain only the characters "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ" 

struct ConnectPayload {
    client_id: u8,
    will_topic: u8,
    will_message: u8,
    username: Option<u8>,
    password: Option<u8>,
}


impl ConnectPayload {
    pub fn new(client_id: u8, will_topic: u8, will_message: u8, username: u8, password: u8) -> Self {
        Self {
            client_id,
            will_topic,
            will_message,
            username: Some(username),
            password: Some(password),
        }
    }
}