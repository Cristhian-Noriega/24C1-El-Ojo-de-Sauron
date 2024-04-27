
struct UnsubscribePayload{
    topic_filters: Vec<TopicFilter>,
}

struct TopicFilter {
    topic: String,
}

impl SubscribePayload {
    pub fn new(topic_filters: Vec<TopicFilter>) -> Self {
        Self {
            topic_filters,
        }
    }
}