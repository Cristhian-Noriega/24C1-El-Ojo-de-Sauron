use {
    errors::error::Error,
    model::{
        components::{
            encoded_string::EncodedString, fixed_header::FixedHeader, login::Login, qos::QoS,
            remaining_length::RemainingLength, topic_filter::TopicFilter, topic_level::TopicLevel,
            topic_name::TopicName, will::Will,
        },
        packets::{
            connack::Connack, connect::Connect, disconnect::Disconnect, pingreq::Pingreq,
            pingresp::Pingresp, puback::Puback, publish::Publish, suback::Suback,
            subscribe::Subscribe, unsuback::Unsuback, unsubscribe::Unsubscribe,
        },
        return_codes::{
            connect_return_code::ConnectReturnCode, suback_return_code::SubackReturnCode,
        },
    },
    std::io::Read,
};

pub mod errors;
pub mod model;

const PROTOCOL_NAME: [u8; 4] = [b'M', b'Q', b'T', b'T'];
const PROTOCOL_LEVEL: u8 = 0x04;
