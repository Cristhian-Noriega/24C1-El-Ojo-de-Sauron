use std::io::{Cursor, Read};

use crate::{
    decrypt, Connack, Connect, Disconnect, Error, FixedHeader, Pingreq, Pingresp, Puback, Publish,
    Suback, Subscribe, Unsuback, Unsubscribe, EXTRA_DATA_SIZE,
};

use super::packets::*;

/// A packet of information that is sent over the network. MQTT has fourteen types of packets.
#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(Puback),
    Subscribe(Subscribe),
    Suback(Suback),
    Unsubscribe(Unsubscribe),
    Unsuback(Unsuback),
    Pingreq(Pingreq),
    Pingresp(Pingresp),
    Disconnect(Disconnect),
}

impl Packet {
    /// Converts a byte stream into an MQTT packet.
    pub fn from_bytes(stream: &mut dyn Read, key: &[u8]) -> Result<Self, Error> {
        let fixed_header = FixedHeader::from_bytes(stream)?;

        let packet_type = fixed_header.first_byte() >> 4;

        let remaining_length_value = fixed_header.remaining_length().value();

        let encrypted_content = &mut vec![0; remaining_length_value + EXTRA_DATA_SIZE];
        stream.read_exact(encrypted_content)?;

        let content = decrypt(encrypted_content, key).unwrap();
        let stream = &mut Cursor::new(content);

        let packet = match packet_type {
            CONNECT_PACKET_TYPE => {
                let connect_packet = Connect::from_bytes(fixed_header, stream)?;

                Packet::Connect(connect_packet)
            }
            CONNACK_PACKET_TYPE => {
                let connack_packet = Connack::from_bytes(fixed_header, stream)?;

                Packet::Connack(connack_packet)
            }
            PUBLISH_PACKET_TYPE => {
                let publish_packet = Publish::from_bytes(fixed_header, stream)?;

                Packet::Publish(publish_packet)
            }
            SUBSCRIBE_PACKET_TYPE => {
                let subscribe_packet = Subscribe::from_bytes(fixed_header, stream)?;

                Packet::Subscribe(subscribe_packet)
            }
            SUBACK_PACKET_TYPE => {
                let suback_packet = Suback::from_bytes(fixed_header, stream)?;
                Packet::Suback(suback_packet)
            }
            PUBACK_PACKET_TYPE => {
                let puback_packet = Puback::from_bytes(fixed_header, stream)?;

                Packet::Puback(puback_packet)
            }
            DISCONNECT_PACKET_TYPE => {
                let disconnect_packet = Disconnect::from_bytes(fixed_header)?;

                Packet::Disconnect(disconnect_packet)
            }
            PINGREQ_PACKET_TYPE => {
                let pingreq_packet = Pingreq::from_bytes(fixed_header)?;
                Packet::Pingreq(pingreq_packet)
            }
            PINGRESP_PACKET_TYPE => {
                let pingresp_packet = Pingresp::from_bytes(fixed_header)?;
                Packet::Pingresp(pingresp_packet)
            }
            UNSUBSCRIBE_PACKET_TYPE => {
                let unsubscribe_packet = Unsubscribe::from_bytes(fixed_header, stream)?;

                Packet::Unsubscribe(unsubscribe_packet)
            }
            UNSUBACK_PACKET_TYPE => {
                let unsuback_packet = Unsuback::from_bytes(fixed_header, stream)?;
                Packet::Unsuback(unsuback_packet)
            }
            _ => return Err(Error::new(format!("Invalid packet type: {}", packet_type))),
        };

        if let Ok(remaining_length) = stream.read(&mut [0; 1]) {
            if remaining_length != 0 {
                return Err(Error::new("Invalid remaining length".to_string()));
            }
        }

        Ok(packet)
    }

    /// Converts the MQTT packet into a byte vector.
    pub fn to_bytes(&self, key: &[u8]) -> Vec<u8> {
        let mut packet_bytes = vec![];

        match self {
            Packet::Connect(connect_packet) => {
                packet_bytes.extend(connect_packet.to_bytes(key));
            }
            Packet::Connack(connack_packet) => {
                packet_bytes.extend(connack_packet.to_bytes(key));
            }
            Packet::Publish(publish_packet) => {
                packet_bytes.extend(publish_packet.to_bytes(key));
            }
            Packet::Subscribe(subscribe_packet) => {
                packet_bytes.extend(subscribe_packet.to_bytes(key));
            }
            Packet::Suback(suback_packet) => {
                packet_bytes.extend(suback_packet.to_bytes(key));
            }
            Packet::Puback(puback_packet) => {
                packet_bytes.extend(puback_packet.to_bytes(key));
            }
            Packet::Disconnect(disconnect_packet) => {
                packet_bytes.extend(disconnect_packet.to_bytes(key));
            }
            Packet::Pingreq(pingreq_packet) => {
                packet_bytes.extend(pingreq_packet.to_bytes(key));
            }
            Packet::Pingresp(pingresp_packet) => {
                packet_bytes.extend(pingresp_packet.to_bytes(key));
            }
            Packet::Unsubscribe(unsubscribe_packet) => {
                packet_bytes.extend(unsubscribe_packet.to_bytes(key));
            }
            Packet::Unsuback(unsuback_packet) => {
                packet_bytes.extend(unsuback_packet.to_bytes(key));
            }
        }
        packet_bytes
    }
}
