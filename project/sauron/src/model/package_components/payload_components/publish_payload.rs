
struct PublshPayload {
    ApplicationMessage: Option<u8>,
}

impl PublshPayload {
    pub fn new(ApplicationMessage: u8) -> Self {
        Self {
            ApplicationMessage: Some(ApplicationMessage),
        }
    }
}
