use crate::Error;

/// Representa los distintos niveles de calidad de servicio (QoS) en MQTT.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum QoS {
    /// Los mensajes se entregan a lo sumo una vez.
    AtMost,
    /// Los mensajes se entregan al menos una vez.
    AtLeast,
    /// Los mensajes se entregan exactamente una vez.
    Exactly,
}

impl QoS {
    /// Convierte el QoS en un byte.
    pub fn to_byte(&self) -> u8 {
        match self {
            QoS::AtMost => 0x00,
            QoS::AtLeast => 0x01,
            QoS::Exactly => 0x02,
        }
    }

    /// Convierte un byte en un QoS.
    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            0x00 => Ok(QoS::AtMost),
            0x01 => Ok(QoS::AtLeast),
            0x02 => Ok(QoS::Exactly),
            _ => Err(Error::new(format!("Invalid QoS: {}", byte))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_to_byte() {
        assert_eq!(QoS::AtMost.to_byte(), 0x00);
        assert_eq!(QoS::AtLeast.to_byte(), 0x01);
        assert_eq!(QoS::Exactly.to_byte(), 0x02);
    }

    #[test]
    fn test_qos_from_byte() {
        assert_eq!(QoS::from_byte(0x00).unwrap(), QoS::AtMost);
        assert_eq!(QoS::from_byte(0x01).unwrap(), QoS::AtLeast);
        assert_eq!(QoS::from_byte(0x02).unwrap(), QoS::Exactly);
    }
}