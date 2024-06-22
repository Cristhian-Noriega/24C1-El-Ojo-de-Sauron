//! This library implements the MQTT protocol version 3.1.1.
//!
//! Its main goal is to provide an interface for the creation and manipulation of MQTT packets.
//!
//! Using from_bytes and to_bytes you can convert the packets to and from bytes, respectively.

use {
    encryptation::{
        encryptation::{decrypt, encrypt},
        EXTRA_DATA_SIZE,
    },
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

/// error handling
pub mod errors;

/// mqtt model
pub mod model;

/// encryptation for packet
mod encryptation;

const PROTOCOL_NAME: [u8; 4] = [b'M', b'Q', b'T', b'T'];
const PROTOCOL_LEVEL: u8 = 0x04;
