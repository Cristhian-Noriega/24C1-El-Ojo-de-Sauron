use std::io::Read;

use crate::{errors::error::Error, Connack, Connect, FixedHeader, Publish, Disconnect, Puback, Pingreq, Pingresp};


pub const CONNECT_PACKET_TYPE: u8 = 0x01;
pub const CONNACK_PACKET_TYPE: u8 = 0x02;
pub const PUBLISH_PACKET_TYPE: u8 = 0x03;
pub const PUBACK_PACKET_TYPE: u8 = 0x04;
pub const PINGREQ_PACKET_TYPE: u8 = 0x12;
pub const PINGRESP_PACKET_TYPE: u8 = 0x13;
pub const DISCONNECT_PACKET_TYPE: u8 = 0x14;

#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(Puback),
    Disconnect(Disconnect),
    Pingreq(Pingreq),
    Pingresp(Pingresp),
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
            PUBACK_PACKET_TYPE => {
                let puback_packet = Publish::from_bytes(fixed_header, stream)?;

                Ok(Packet::Publish(puback_packet))
            }
            DISCONNECT_PACKET_TYPE => {
                let disconnect_packet = Disconnect::from_bytes(fixed_header)?;

                Ok(Packet::Disconnect(disconnect_packet))
            }
            PINGREQ_PACKET_TYPE => {
                let pingreq_packet = Pingreq::from_bytes(fixed_header)?;
                Ok(Packet::Pingreq(pingreq_packet))
            }
            PINGRESP_PACKET_TYPE => {
                let pingresp_packet = Pingresp::from_bytes(fixed_header)?;
                Ok(Packet::Pingresp(pingresp_packet))
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
            Packet::Puback(puback_packet) => {
                packet_bytes.push(PUBACK_PACKET_TYPE);
                packet_bytes.extend(puback_packet.to_bytes());
            }
            Packet::Disconnect(disconnect_packet) => {
                packet_bytes.push(DISCONNECT_PACKET_TYPE);
                packet_bytes.extend(disconnect_packet.to_bytes());
            }
            Packet::Pingreq(pingreq_packet) => {
                packet_bytes.push(PINGREQ_PACKET_TYPE);
                packet_bytes.extend(pingreq_packet.to_bytes());
            }
            Packet::Pingresp(pingresp_packet) => {
                packet_bytes.push(PINGRESP_PACKET_TYPE);
                packet_bytes.extend(pingresp_packet.to_bytes());
            }
        }
        packet_bytes
    }
}
