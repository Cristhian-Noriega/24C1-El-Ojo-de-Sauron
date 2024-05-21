pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod pingreq;
pub mod pingresp;
pub mod puback;
pub mod publish;
pub mod suback;
pub mod subscribe;
pub mod unsuback;
pub mod unsubscribe;

pub const CONNECT_PACKET_TYPE: u8 = 0x01;
pub const CONNACK_PACKET_TYPE: u8 = 0x02;
pub const PUBLISH_PACKET_TYPE: u8 = 0x03;
pub const PUBACK_PACKET_TYPE: u8 = 0x04;
pub const SUBSCRIBE_PACKET_TYPE: u8 = 0x08;
pub const SUBACK_PACKET_TYPE: u8 = 0x09;
pub const PINGREQ_PACKET_TYPE: u8 = 0x12;
pub const PINGRESP_PACKET_TYPE: u8 = 0x13;
pub const DISCONNECT_PACKET_TYPE: u8 = 0x14;
pub const UNSUBSCRIBE_PACKET_TYPE: u8 = 0x0A;
pub const UNSUBACK_PACKET_TYPE: u8 = 0x0B;

const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x00;

const DEFAULT_VARIABLE_HEADER_LENGTH: usize = 2;
