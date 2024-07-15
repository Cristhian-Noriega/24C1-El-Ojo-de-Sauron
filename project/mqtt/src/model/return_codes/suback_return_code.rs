use crate::{MqttError, MqttResult};

/// Represents the different return codes of a Suback in MQTT.
#[derive(PartialEq, Debug)]
pub enum SubackReturnCode {
    SuccessMaximumQoS0,
    SuccessMaximumQoS1,
    SuccessMaximumQoS2,
    Failure,
}

impl SubackReturnCode {
    /// Converts the return code of a Suback to a byte.
    pub fn to_byte(&self) -> u8 {
        match self {
            SubackReturnCode::SuccessMaximumQoS0 => 0x00,
            SubackReturnCode::SuccessMaximumQoS1 => 0x01,
            SubackReturnCode::SuccessMaximumQoS2 => 0x02,
            SubackReturnCode::Failure => 0x80,
        }
    }

    /// Converts a byte into a Suback return code.
    pub fn from_byte(byte: u8) -> MqttResult<Self> {
        match byte {
            0x00 => Ok(SubackReturnCode::SuccessMaximumQoS0),
            0x01 => Ok(SubackReturnCode::SuccessMaximumQoS1),
            0x02 => Ok(SubackReturnCode::SuccessMaximumQoS2),
            0x80 => Ok(SubackReturnCode::Failure),
            _ => Err(MqttError::InvalidReturnCode(format!(
                "Invalid SubackReturnCode: {}",
                byte
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suback_return_code_to_byte() {
        assert_eq!(SubackReturnCode::SuccessMaximumQoS0.to_byte(), 0x00);
        assert_eq!(SubackReturnCode::SuccessMaximumQoS1.to_byte(), 0x01);
        assert_eq!(SubackReturnCode::SuccessMaximumQoS2.to_byte(), 0x02);
        assert_eq!(SubackReturnCode::Failure.to_byte(), 0x80);
    }

    #[test]
    fn test_suback_return_code_from_byte() {
        assert_eq!(
            SubackReturnCode::from_byte(0x00).unwrap(),
            SubackReturnCode::SuccessMaximumQoS0
        );
        assert_eq!(
            SubackReturnCode::from_byte(0x01).unwrap(),
            SubackReturnCode::SuccessMaximumQoS1
        );
        assert_eq!(
            SubackReturnCode::from_byte(0x02).unwrap(),
            SubackReturnCode::SuccessMaximumQoS2
        );
        assert_eq!(
            SubackReturnCode::from_byte(0x80).unwrap(),
            SubackReturnCode::Failure
        );
        assert!(SubackReturnCode::from_byte(0x03).is_err());
    }
}
