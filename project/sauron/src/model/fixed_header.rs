use std::io::Read;

use crate::errors::error::Error;

const FIXED_HEADER_LENGTH: usize = 2;

pub struct FixedHeader {
    pub first_byte: u8,
    pub remaining_length: u8, // Cambiar por remaining_length real
}

impl FixedHeader {
    pub fn new(first_byte: u8, remaining_length: u8) -> FixedHeader {
        FixedHeader {
            first_byte,
            remaining_length,
        }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<FixedHeader, Error> {
        let buffer = &mut [0; FIXED_HEADER_LENGTH];
        stream.read_exact(buffer)?;

        Ok(FixedHeader {
            first_byte: buffer[0],
            remaining_length: buffer[1],
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.first_byte, self.remaining_length]
    }

    pub fn first_byte(&self) -> u8 {
        self.first_byte
    }

    pub fn remaining_length(&self) -> u8 {
        self.remaining_length
    }
}
