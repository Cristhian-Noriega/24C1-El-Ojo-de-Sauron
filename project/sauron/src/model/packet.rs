use std::io::Read;

use crate::{errors::error::Error, Connack, Connect, FixedHeader, Publish, Subscribe};

pub const CONNECT_PACKET_TYPE: u8 = 0x01;
pub const CONNACK_PACKET_TYPE: u8 = 0x02;
pub const PUBLISH_PACKET_TYPE: u8 = 0x03;
pub const SUBSCRIBE_PACKET_TYPE: u8 = 0x08;

#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Subscribe(Subscribe),
}

impl Packet {
    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let fixed_header = FixedHeader::from_bytes(stream)?;

        let packet_type = fixed_header.first_byte() >> 4;

        match packet_type {
            CONNECT_PACKET_TYPE => {
                let connect_packet = Connect::from_bytes(fixed_header, stream)?;

                Ok(Packet::Connect(connect_packet))
            }
            CONNACK_PACKET_TYPE => {
                let connack_packet = Connack::from_bytes(fixed_header, stream)?;

                Ok(Packet::Connack(connack_packet))
            }
            PUBLISH_PACKET_TYPE => {
                let publish_packet = Publish::from_bytes(fixed_header, stream)?;

                Ok(Packet::Publish(publish_packet))
            }
            SUBSCRIBE_PACKET_TYPE => {
                let subscribe_packet = Subscribe::from_bytes(fixed_header, stream)?;

                Ok(Packet::Subscribe(subscribe_packet))
            }
            _ => Err(crate::errors::error::Error::new(format!(
                "Invalid packet type: {}",
                packet_type
            ))),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut packet_bytes = vec![];

        match self {
            Packet::Connect(connect_packet) => {
                packet_bytes.push(CONNECT_PACKET_TYPE);
                packet_bytes.extend(connect_packet.to_bytes());
            }
            Packet::Connack(connack_packet) => {
                packet_bytes.push(CONNACK_PACKET_TYPE);
                packet_bytes.extend(connack_packet.to_bytes());
            }
            Packet::Publish(publish_packet) => {
                packet_bytes.push(PUBLISH_PACKET_TYPE);
                packet_bytes.extend(publish_packet.to_bytes());
            }
            Packet::Subscribe(subscribe_packet) => {
                packet_bytes.push(SUBSCRIBE_PACKET_TYPE);
                packet_bytes.extend(subscribe_packet.to_bytes());
            }
        }
        packet_bytes
    }
}
