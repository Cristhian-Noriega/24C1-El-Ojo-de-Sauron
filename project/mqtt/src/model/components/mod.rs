/// convertir string a bytes
pub mod encoded_string;
/// fixed header de los packets
pub mod fixed_header;
/// login
pub mod login;
/// calidad de servicio
pub mod qos;
/// calculo del remaining length
pub mod remaining_length;
/// filtro de topic
pub mod topic_filter;
/// nivel de topic
pub mod topic_level;
/// nombre del topic
pub mod topic_name;
/// will
pub mod will;

const FORWARD_SLASH: u8 = 0x2F;
const SERVER_RESERVED: u8 = 0x24;
