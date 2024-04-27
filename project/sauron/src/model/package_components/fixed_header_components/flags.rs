use crate::{errors::error::Error, model::package_components::fixed_header_components::qos::QoS};

pub struct Flags {
    retain: bool,
    dup: bool,
    qos: QoS,
}

impl Flags {
    pub fn new(retain: bool, dup: bool, qos: QoS) -> Self {
        Self { retain, dup, qos }
    }

    pub fn into_u8(self) -> u8 {
        let mut flags: u8 = 0x00;

        if self.retain {
            flags |= 0x01 << 0;
        }

        if self.dup {
            flags |= 0x01 << 3;
        }

        flags |= self.qos.into_u8() << 1;

        flags
    }

    pub fn from_u8(value: u8) -> Result<Self, Error> {
        let retain = (value & 0x01) == 0x01;
        let dup = (value & 0x08) == 0x08;
        let qos = QoS::from_u8((value & 0x06) >> 1)?;

        Ok(Self { retain, dup, qos })
    }
}
