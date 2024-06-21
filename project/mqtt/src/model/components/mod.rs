/// manage encoded strings
pub mod encoded_string;
/// fixed header of MQTT packets
pub mod fixed_header;
/// login
pub mod login;
/// quality of service
pub mod qos;
/// calculate the remaining length
pub mod remaining_length;
/// topic filter
pub mod topic_filter;
/// topic levels
pub mod topic_level;
/// topic name
pub mod topic_name;
/// will
pub mod will;

const FORWARD_SLASH: u8 = 0x2F;
const SERVER_RESERVED: u8 = 0x24;
