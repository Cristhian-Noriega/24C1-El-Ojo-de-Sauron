
struct SubAckPayload {
    return_codes: Vec<SubAckReturnCode>,
}

struct SubAckReturnCode {
    return_code: u8,
}

impl SubAckPayload {
    pub fn new(return_codes: Vec<SubAckReturnCode>) -> Self {
        Self {
            return_codes,
        }
    }
}
