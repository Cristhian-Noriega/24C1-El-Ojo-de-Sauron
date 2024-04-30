use crate::{errors::error::Error, model::package_components::fixed_header_components::qos::QoS};

pub struct FixedHeaderFlagsPublish {
    retain: bool,
    dup: bool,
    qos: QoS,
}

impl FixedHeaderFlagsPublish {
    pub fn new(retain: bool, dup: bool, qos: QoS) -> Self {
        Self { retain, dup, qos }
    }

    pub fn into_byte(&self) -> u8 {
        let mut flags: u8 = 0x00;

        if self.retain {
            flags |= 0x01 << 0;
        }

        if self.dup {
            flags |= 0x01 << 3;
        }

        flags |= self.qos.into_byte() << 1;

        flags
    }

    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        // ----
        let retain = (byte & 0x01) != 0;
        let qos = QoS::from_byte((byte & 0x06) >> 1)?;
        let dup = (byte & 0x08) != 0;

        Ok(Self { retain, dup, qos })
    }
}
