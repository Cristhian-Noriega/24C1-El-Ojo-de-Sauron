use crate::{errors::error::Error, ConnectPacket};

const CONNECT_PACKET_TYPE: u8 = 0x01;

#[derive(Debug)]
pub enum Packet {
    Connect(ConnectPacket),
}

impl Packet {
    pub fn from_bytes(stream: &mut std::io::Cursor<Vec<u8>>) -> Result<Self, Error> {
        let packet_type = stream.get_ref()[0] >> 4;

        match packet_type {
            CONNECT_PACKET_TYPE => {
                let connect_packet = ConnectPacket::from_bytes(stream)?;

                Ok(Packet::Connect(connect_packet))
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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::packet::Packet;

    #[test]
    fn test_packet_from_bytes() {
        let mut stream = std::io::Cursor::new(vec![
            0x10, 0x0e, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x00, 0x00, 0x00, 0x00,
        ]);

        assert!(Packet::from_bytes(&mut stream).is_ok());
    }
}
