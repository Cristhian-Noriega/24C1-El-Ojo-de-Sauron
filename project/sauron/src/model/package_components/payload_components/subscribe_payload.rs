

struct SubscribePayload{
    topic_filters: Vec<TopicFilter>,
}

struct TopicFilter {
    topic: String,
    qos: u8,
}

impl SubscribePayload {
    pub fn new(topic_filters: Vec<TopicFilter>) -> Self {
        Self {
            topic_filters,
        }
    }
}