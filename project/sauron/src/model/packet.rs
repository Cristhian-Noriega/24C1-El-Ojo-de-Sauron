use crate::{errors::error::Error, ConnectPacket, ConnackPacket};

pub const CONNECT_PACKET_TYPE: u8 = 0x01;
pub const CONNACK_PACKET_TYPE: u8 = 0x02;

#[derive(Debug)]
pub enum Packet {
    Connect(ConnectPacket),
    Connack(ConnackPacket)
}

impl Packet {
    pub fn from_bytes(stream: &mut std::io::Cursor<Vec<u8>>) -> Result<Self, Error> {
        let packet_type = stream.get_ref()[0] >> 4;

        match packet_type {
            CONNECT_PACKET_TYPE => {
                let connect_packet = ConnectPacket::from_bytes(stream)?;

                Ok(Packet::Connect(connect_packet))
            }
            CONNACK_PACKET_TYPE => {
                let connack_packet = ConnackPacket::from_bytes(stream)?;

                Ok(Packet::Connack(connack_packet))
            }
            _ => Err(crate::errors::error::Error::new(format!(
                "Invalid packet type: {}",
                packet_type
            ))),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Packet::Connect(connect_packet) => {
                let mut packet_bytes = vec![];

                packet_bytes.push(0x10);
                packet_bytes.extend(connect_packet.to_bytes());

                packet_bytes
            }
            Packet::Connack(connack_packet) => {
                let mut packet_bytes = vec![];

                packet_bytes.push(0x20);
                packet_bytes.extend(connack_packet.to_bytes());

                packet_bytes
            }
        }
    }
}
