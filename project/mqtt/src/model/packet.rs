use std::io::{Cursor, Read};

use crate::{
    errors::error::Error, Connack, Connect, Disconnect, FixedHeader, Pingreq, Pingresp, Puback,
    Publish, Suback, Subscribe, Unsuback, Unsubscribe,
};

use super::packets::*;

/// Un paquete de informacion que es enviado a traves de la red. MQTT tiene catorce tipos de paquetes.
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
    /// Convierte un stream de bytes en un paquete MQTT.
    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let fixed_header = FixedHeader::from_bytes(stream)?;

        let packet_type = fixed_header.first_byte() >> 4;

        let remaining_length_value = fixed_header.remaining_length().value();

        let content = &mut vec![0; remaining_length_value];
        stream.read_exact(content)?;

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

    /// Convierte el paquete MQTT en un vector de bytes.
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
            Packet::Suback(suback_packet) => {
                packet_bytes.push(SUBACK_PACKET_TYPE);
                packet_bytes.extend(suback_packet.to_bytes());
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
            Packet::Unsubscribe(unsubscribe_packet) => {
                packet_bytes.push(UNSUBSCRIBE_PACKET_TYPE);
                packet_bytes.extend(unsubscribe_packet.to_bytes());
            }
            Packet::Unsuback(unsuback_packet) => {
                packet_bytes.push(UNSUBACK_PACKET_TYPE);
                packet_bytes.extend(unsuback_packet.to_bytes());
            }
        }
        packet_bytes
    }
}
