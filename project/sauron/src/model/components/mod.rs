pub mod encoded_string;
pub mod fixed_header;
pub mod login;
pub mod qos;
pub mod remaining_length;
pub mod topic_filter;
pub mod topic_level;
pub mod topic_name;
pub mod will;

const FORWARD_SLASH: u8 = 0x2F;
const SERVER_RESERVED: u8 = 0x24;
