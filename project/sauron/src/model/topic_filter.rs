use std::io::Read;

use crate::{errors::error::Error, model::encoded_string::EncodedString};

use super::{topic_level::TopicLevel, topic_name::TopicName};

const FORWARD_SLASH: u8 = 0x2F;

#[derive(Debug)]
pub struct TopicFilter {
    levels: Vec<TopicLevel>,
}

impl TopicFilter {
    pub fn new(levels: Vec<TopicLevel>) -> Self {
        Self { levels }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let encoded_string_topic_filter = EncodedString::from_bytes(stream)?;
        let bytes = encoded_string_topic_filter.content();

        if bytes.is_empty() {
            return Err(Error::new("Invalid topic name".to_string()));
        }

        let mut levels = vec![];

        let levels_bytes: Vec<Vec<u8>> = bytes
            .split(|&byte| byte == FORWARD_SLASH)
            .map(|slice| slice.to_vec())
            .collect();

        for (level_index, level_byte) in levels_bytes.iter().enumerate() {
            let topic_level = TopicLevel::from_bytes(level_byte.to_vec())?;

            if let TopicLevel::MultiLevelWildcard = topic_level {
                if level_index != levels_bytes.len() - 1 {
                    return Err(Error::new(
                        "Multi-level wildcard must be the last level".to_string(),
                    ));
                }
            }

            levels.push(topic_level);
        }

        Ok(Self { levels })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let topic_bytes = self
            .levels
            .iter()
            .map(|level| level.to_bytes())
            .chain(std::iter::once(vec![FORWARD_SLASH]))
            .flatten()
            .collect();

        EncodedString::new(topic_bytes).to_bytes()
    }

    pub fn match_topic_name(&self, topic_name: TopicName) -> bool {
        let name_levels = topic_name.levels();
        let filter_levels = &self.levels;

        for (level_index, filter_level) in filter_levels.iter().enumerate() {
            match filter_level {
                TopicLevel::MultiLevelWildcard => return true,
                TopicLevel::SingleLevelWildcard => continue,
                TopicLevel::Literal(filter_level) => match name_levels.get(level_index) {
                    Some(name_level) => {
                        if filter_level != name_level {
                            return false;
                        }
                    }

                    None => return false,
                },
            }
        }

        filter_levels.len() == name_levels.len()
    }

    pub fn length(&self) -> usize {
        self.to_bytes().len()
    }
}

mod test {
    use super::*;
    use crate::model::encoded_string::EncodedString;
    use std::io::Cursor;

    #[allow(dead_code)]
    fn from_slice(bytes: &[u8]) -> impl Read {
        let encoded_string = EncodedString::new(bytes.to_vec());
        Cursor::new(encoded_string.to_bytes())
    }

    #[test]
    fn test_valid_topic_filter() {
        let bytes = &mut from_slice(b"home/livingroom");
        assert!(TopicFilter::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"home/living room");
        assert!(TopicFilter::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"home/+/living-room");
        assert!(TopicFilter::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"home/+/living-room/#");
        assert!(TopicFilter::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"+/+/#");
        assert!(TopicFilter::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"+");
        assert!(TopicFilter::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"#");
        assert!(TopicFilter::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"/");
        assert!(TopicFilter::from_bytes(bytes).is_ok());
    }

    #[test]
    fn test_invalid_topic_filter() {
        let bytes = &mut from_slice(b"home+");
        assert!(TopicFilter::from_bytes(bytes).is_err());

        let bytes = &mut from_slice(b"#/livingroom");
        assert!(TopicFilter::from_bytes(bytes).is_err());

        let bytes = &mut from_slice(b"home#");
        assert!(TopicFilter::from_bytes(bytes).is_err());

        let bytes = &mut from_slice(b"home/#/livingroom");
        assert!(TopicFilter::from_bytes(bytes).is_err());
    }

    #[test]
    fn test_length() {
        let bytes = &mut from_slice(b"home/livingroom");
        let topic_filter = TopicFilter::from_bytes(bytes).unwrap();

        assert_eq!(topic_filter.length(), 17);

        let bytes = &mut from_slice(b"/");
        let topic_filter = TopicFilter::from_bytes(bytes).unwrap();

        assert_eq!(topic_filter.length(), 3);
    }

    #[test]
    fn test_matching_topic_names() {
        {
            let filter_bytes = &mut from_slice(b"home/livingroom");
            let name_bytes = &mut from_slice(b"home/livingroom");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name = TopicName::from_bytes(name_bytes).unwrap();

            assert!(topic_filter.match_topic_name(topic_name));
        }

        {
            let filter_bytes = &mut from_slice(b"home/+");
            let name_bytes1 = &mut from_slice(b"home/livingroom");
            let name_bytes2 = &mut from_slice(b"home/kitchen");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name1 = TopicName::from_bytes(name_bytes1).unwrap();
            let topic_name2 = TopicName::from_bytes(name_bytes2).unwrap();

            assert!(topic_filter.match_topic_name(topic_name1));
            assert!(topic_filter.match_topic_name(topic_name2));
        }

        {
            let filter_bytes = &mut from_slice(b"home/+/table");
            let name_bytes1 = &mut from_slice(b"home/livingroom/table");
            let name_bytes2 = &mut from_slice(b"home/kitchen/table");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name1 = TopicName::from_bytes(name_bytes1).unwrap();
            let topic_name2 = TopicName::from_bytes(name_bytes2).unwrap();

            assert!(topic_filter.match_topic_name(topic_name1));
            assert!(topic_filter.match_topic_name(topic_name2));
        }

        {
            let filter_bytes = &mut from_slice(b"home/#");
            let name_bytes1 = &mut from_slice(b"home/livingroom");
            let name_bytes2 = &mut from_slice(b"home/kitchen");
            let name_bytes3 = &mut from_slice(b"home/livingroom/table");
            let name_bytes4 = &mut from_slice(b"home/");
            let name_bytes5 = &mut from_slice(b"home");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name1 = TopicName::from_bytes(name_bytes1).unwrap();
            let topic_name2 = TopicName::from_bytes(name_bytes2).unwrap();
            let topic_name3 = TopicName::from_bytes(name_bytes3).unwrap();
            let topic_name4 = TopicName::from_bytes(name_bytes4).unwrap();
            let topic_name5 = TopicName::from_bytes(name_bytes5).unwrap();

            assert!(topic_filter.match_topic_name(topic_name1));
            assert!(topic_filter.match_topic_name(topic_name2));
            assert!(topic_filter.match_topic_name(topic_name3));
            assert!(topic_filter.match_topic_name(topic_name4));
            assert!(topic_filter.match_topic_name(topic_name5));
        }
        {
            let filter_bytes = &mut from_slice(b"+");
            let name_bytes = &mut from_slice(b"home");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name = TopicName::from_bytes(name_bytes).unwrap();

            assert!(topic_filter.match_topic_name(topic_name));
        }
        {
            let filter_bytes = &mut from_slice(b"#");
            let name_bytes1 = &mut from_slice(b"home");
            let name_bytes2 = &mut from_slice(b"home/livingroom");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name1 = TopicName::from_bytes(name_bytes1).unwrap();
            let topic_name2 = TopicName::from_bytes(name_bytes2).unwrap();

            assert!(topic_filter.match_topic_name(topic_name1));
            assert!(topic_filter.match_topic_name(topic_name2));
        }
        {
            let filter_bytes = &mut from_slice(b"+/+");
            let name_bytes1 = &mut from_slice(b"home/livingroom");
            let name_bytes2 = &mut from_slice(b"/kitchen");
            let name_bytes3 = &mut from_slice(b"home/");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name1 = TopicName::from_bytes(name_bytes1).unwrap();
            let topic_name2 = TopicName::from_bytes(name_bytes2).unwrap();
            let topic_name3 = TopicName::from_bytes(name_bytes3).unwrap();

            assert!(topic_filter.match_topic_name(topic_name1));
            assert!(topic_filter.match_topic_name(topic_name2));
            assert!(topic_filter.match_topic_name(topic_name3));
        }
    }

    #[test]
    fn test_rejecting_topic_names() {
        {
            let filter_bytes = &mut from_slice(b"home/livingroom");
            let name_bytes = &mut from_slice(b"home/kitchen");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name = TopicName::from_bytes(name_bytes).unwrap();

            assert!(!topic_filter.match_topic_name(topic_name));
        }

        {
            let filter_bytes = &mut from_slice(b"home/+");
            let name_bytes = &mut from_slice(b"home/livingroom/table");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name = TopicName::from_bytes(name_bytes).unwrap();

            assert!(!topic_filter.match_topic_name(topic_name));
        }

        {
            let filter_bytes = &mut from_slice(b"home/+/table");
            let name_bytes = &mut from_slice(b"home/table");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name = TopicName::from_bytes(name_bytes).unwrap();

            assert!(!topic_filter.match_topic_name(topic_name));
        }

        {
            let filter_bytes = &mut from_slice(b"home/#");
            let name_bytes = &mut from_slice(b"work");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name = TopicName::from_bytes(name_bytes).unwrap();

            assert!(!topic_filter.match_topic_name(topic_name));
        }

        {
            let filter_bytes = &mut from_slice(b"+");
            let name_bytes1 = &mut from_slice(b"home/livingroom");
            let name_bytes2 = &mut from_slice(b"/livingroom");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name1 = TopicName::from_bytes(name_bytes1).unwrap();
            let topic_name2 = TopicName::from_bytes(name_bytes2).unwrap();

            assert!(!topic_filter.match_topic_name(topic_name1));
            assert!(!topic_filter.match_topic_name(topic_name2));
        }

        {
            let filter_bytes = &mut from_slice(b"+/+");
            let name_bytes = &mut from_slice(b"livingroom");

            let topic_filter = TopicFilter::from_bytes(filter_bytes).unwrap();
            let topic_name = TopicName::from_bytes(name_bytes).unwrap();

            assert!(!topic_filter.match_topic_name(topic_name));
        }
    }
}
