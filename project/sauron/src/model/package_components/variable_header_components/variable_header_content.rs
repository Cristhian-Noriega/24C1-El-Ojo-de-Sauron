use super::contents::connect_variable_header::ConnectVariableHeader;

pub enum VariableHeaderContent {
    Connect(ConnectVariableHeader),
    // ConnAck(),
    // Publish(),
    // PubAck(),
    //PubRec(),
    //PubRel(),
    //PubComp(),
    // Subscribe(),
    // SubAck(),
    // Unsubscribe(),
    // UnsubAck(),
    //PingReq(),
    //PingResp(),
    // Disconnect(),
}

impl VariableHeaderContent {
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            VariableHeaderContent::Connect(connect) => connect.into_bytes(),
        }
    }

    pub fn get_length(self) -> usize {
        match self {
            VariableHeaderContent::Connect(connect) => connect.get_length(),
        }
    }
}
