pub enum FixedHeaderFlags {
    Connect(FixedHeaderFlagsConnect),
}

impl FixedHeaderFlags {
    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        todo!()
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            FixedHeaderFlags::Connect(fixed_header_flags_connect) => {
                fixed_header_flags_connect.to_u8()
            }
        }
    }
}
