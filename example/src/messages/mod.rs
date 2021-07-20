#[allow(dead_code)]
pub mod biocells;
#[allow(dead_code)]
pub mod scales;
#[allow(dead_code)]
pub mod trucks;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelReference{
    channel_address: String,
    msg_id: String,
}

impl ChannelReference{
    pub fn new(channel_id: String, announce_id: String, msg_id: String) -> Self {
        ChannelReference {
            channel_address: format!("{}:{}", channel_id, announce_id),
            msg_id
        }
    }
}
