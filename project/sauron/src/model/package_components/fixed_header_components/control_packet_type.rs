use crate::errors::error::Error;
pub enum ControlPacketType {
    Connect,
    // ConnAck,
    // Publish,
    // PubAck,
    // PubRec,
    // PubRel,
    // PubComp,
    // Subscribe,
    // SubAck,
    // Unsubscribe,
    // UnsubAck,
    // PingReq,
    // PingResp,
    // Disconnect,
}

impl ControlPacketType {
    pub fn into_u8(self) -> u8 {
        match self {
            ControlPacketType::Connect => 0x01,
            // ControlPacketType::ConnAck => 0x02,
            // ControlPacketType::Publish => 0x03,
            // ControlPacketType::PubAck => 0x04,
            // ControlPacketType::PubRec => 0x05,
            // ControlPacketType::PubRel => 0x06,
            // ControlPacketType::PubComp => 0x07,
            // ControlPacketType::Subscribe => 0x08,
            // ControlPacketType::SubAck => 0x09,
            // ControlPacketType::Unsubscribe => 0x0A,
            // ControlPacketType::UnsubAck => 0x0B,
            // ControlPacketType::PingReq => 0x0C,
            // ControlPacketType::PingResp => 0x0D,
            // ControlPacketType::Disconnect => 0x0E,
        }
    }

    pub fn from_u8(value: u8) -> Result<Self, Error> {
        match value {
            0x01 => Ok(ControlPacketType::Connect),
            // 0x02 => Ok(ControlPacketType::ConnAck),
            // 0x03 => Ok(ControlPacketType::Publish),
            // 0x04 => Ok(ControlPacketType::PubAck),
            // 0x05 => Ok(ControlPacketType::PubRec),
            // 0x06 => Ok(ControlPacketType::PubRel),
            // 0x07 => Ok(ControlPacketType::PubComp),
            // 0x08 => Ok(ControlPacketType::Subscribe),
            // 0x09 => Ok(ControlPacketType::SubAck),
            // 0x0A => Ok(ControlPacketType::Unsubscribe),
            // 0x0B => Ok(ControlPacketType::UnsubAck),
            // 0x0C => Ok(ControlPacketType::PingReq),
            // 0x0D => Ok(ControlPacketType::PingResp),
            // 0x0E => Ok(ControlPacketType::Disconnect),
            _ => Err(Error::new(format!(
                "Invalid Control Packet Type: {}",
                value
            ))),
        }
    }
}
