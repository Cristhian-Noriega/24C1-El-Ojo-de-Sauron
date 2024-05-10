use std::{io::Read, vec};

use crate::errors::error::Error;

use super::remaining_length::RemainingLength;

pub struct FixedHeader {
    pub first_byte: u8,
    pub remaining_length: RemainingLength,
}

impl FixedHeader {
    pub fn new(first_byte: u8, remaining_length: RemainingLength) -> FixedHeader {
        FixedHeader {
            first_byte,
            remaining_length,
        }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<FixedHeader, Error> {
        let first_byte_buffer = &mut [0; 1];
        stream.read_exact(first_byte_buffer)?;

        let first_byte = first_byte_buffer[0];
        let remaining_length = RemainingLength::from_bytes(stream)?;

        Ok(FixedHeader {
            first_byte,
            remaining_length,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut fixed_header_bytes = vec![self.first_byte];
        fixed_header_bytes.extend(self.remaining_length.to_bytes());

        fixed_header_bytes
    }

    pub fn first_byte(&self) -> u8 {
        self.first_byte
    }

    pub fn remaining_length(&self) -> &RemainingLength {
        &self.remaining_length
    }
}
