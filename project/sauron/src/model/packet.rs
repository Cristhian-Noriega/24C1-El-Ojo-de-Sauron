use crate::{errors::error::Error, Connack, Connect};

pub const CONNECT_PACKET_TYPE: u8 = 0x01;
pub const CONNACK_PACKET_TYPE: u8 = 0x02;

#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    Connack(Connack),
}

impl Packet {
    pub fn from_bytes(stream: &mut std::io::Cursor<Vec<u8>>) -> Result<Self, Error> {
        let packet_type = stream.get_ref()[0] >> 4;

        match packet_type {
            CONNECT_PACKET_TYPE => {
                let connect_packet = Connect::from_bytes(stream)?;

                Ok(Packet::Connect(connect_packet))
            }
            CONNACK_PACKET_TYPE => {
                let connack_packet = Connack::from_bytes(stream)?;

                Ok(Packet::Connack(connack_packet))
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
                packet_bytes.push(0x10);
                packet_bytes.extend(connect_packet.to_bytes());
            }
            Packet::Connack(connack_packet) => {
                packet_bytes.push(0x20);
                packet_bytes.extend(connack_packet.to_bytes());
            }
        }
        packet_bytes
    }
}
