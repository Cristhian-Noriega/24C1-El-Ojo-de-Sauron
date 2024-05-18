use {
    errors::error::Error,
    model::{
        components::{
            encoded_string::EncodedString, fixed_header::FixedHeader, login::Login, qos::QoS,
            remaining_length::RemainingLength, topic_filter::TopicFilter, topic_level::TopicLevel,
            topic_name::TopicName, will::Will,
        },
        packets::{
            connack::Connack, connect::Connect, disconnect::Disconnect, pingreq::Pingreq,
            pingresp::Pingresp, puback::Puback, publish::Publish, suback::Suback,
            subscribe::Subscribe, unsuback::Unsuback, unsubscribe::Unsubscribe,
        },
        return_codes::{
            connack_return_code::ConnackReturnCode, suback_return_code::SubackReturnCode,
        },
    },
    std::io::Read,
};

pub mod errors;
pub mod model;

const PROTOCOL_NAME: [u8; 4] = [b'M', b'Q', b'T', b'T'];
const PROTOCOL_LEVEL: u8 = 0x04;

#[cfg(test)]
mod test {
    use std::io::Write;
    use std::net::TcpStream;

    use self::model::packet::Packet;

    use super::*;
    const ADDRESS: &str = "test.mosquitto.org:1883";

    #[test]
    fn test_connect() {
        let mut stream = TcpStream::connect(ADDRESS).unwrap();

        // connect packet

        let clean_session = true;
        let keep_alive = 30;
        let client_id = EncodedString::new(b"test".to_vec());
        let will = None;
        let login = None;

        let connect = Connect::new(clean_session, keep_alive, client_id, will, login);

        let connect_bytes = connect.to_bytes();

        stream.write(&connect_bytes).unwrap();

        let packet = Packet::from_bytes(&mut stream).unwrap();

        match packet {
            Packet::Connack(connack) => {
                assert_eq!(connack.session_present(), false);
                assert_eq!(
                    connack.connect_return_code(),
                    &ConnackReturnCode::ConnectionAccepted
                );
            }
            _ => panic!("Invalid packet"),
        }
    }
}
