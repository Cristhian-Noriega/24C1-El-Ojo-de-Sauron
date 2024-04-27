use crate::model::package_components::fixed_header_components::qos::QoS;

pub struct Flags {
    retain: bool,
    dup: bool,
    qos: QoS,
}

impl Flags {
    pub fn new(retain: bool, dup: bool, qos: QoS) -> Self {
        Self { retain, dup, qos }
    }
}
