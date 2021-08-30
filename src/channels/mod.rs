use serde::{Serialize, Deserialize};
use iota_streams_lib::channels::{ChannelWriter, ChannelReader};

pub mod root_channel;
mod category_channel;
mod actor_channel;
pub use actor_channel::DailyChannelManager;
pub mod daily_channel;
pub use category_channel::ActorChannelMsg as ActorChannelInfo;
pub use actor_channel::DailyChannelMsg as DailyChannelInfo;
use std::collections::HashMap;
use serde_json::Value;
use iota_streams_lib::payload::payload_serializers::JsonPacket;
use crate::utils::current_time_secs;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Category{
    Trucks,
    Scales,
    BioCells
}

impl Category{
    pub fn is_trucks(&self) -> bool{
        self == &Category::Trucks
    }
    pub fn is_scales(&self) -> bool{
        self == &Category::Scales
    }
    pub fn is_biocells(&self) -> bool{
        self == &Category::BioCells
    }
    /*pub fn equals_to(&self, other: &Category) -> bool{
        match (self, other) {
            (Category::Trucks, Category::Trucks) => true,
            (Category::Scales, Category::Scales) => true,
            (Category::BioCells, Category::BioCells) => true,
            (_, _) => false
        }
    }*/

    pub fn to_string(&self) -> String{
        match self{
            Category::Trucks => "trucks".to_string(),
            Category::Scales => "weighing_scales".to_string(),
            Category::BioCells => "biocells".to_string()
        }
    }

    pub fn from_string(category: &str) -> Option<Category>{
        let category = category.to_lowercase();
        match category.as_str(){
            "trucks" => Some(Category::Trucks),
            "weighing_scales" => Some(Category::Scales),
            "biocells" => Some(Category::BioCells),
            _ => None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelInfo{
    channel_id: String,
    announce_id: String,
}

impl ChannelInfo{
    pub fn new(channel_id: String, announce_id: String) -> Self {
        ChannelInfo { channel_id, announce_id }
    }

    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }
    pub fn announce_id(&self) -> &str {
        &self.announce_id
    }
    pub fn explorer_url(&self) -> String{
        format!("https://streams-chrysalis-explorer.netlify.app/channel/{}", self.to_string())
    }
    pub fn to_string(&self) -> String{
        format!("{}:{}", self.channel_id, self.announce_id)
    }
}

pub struct MessageReader{
    reader: ChannelReader,
    msgs: Vec<HashMap<String, Value>>,
    last_update: i64,
}

impl MessageReader{
    pub async fn new(channel_info: &ChannelInfo, mainnet: bool) -> anyhow::Result<Self> {
        let mut reader = create_reader(channel_info.channel_id(), channel_info.announce_id(), mainnet);
        reader.attach().await?;
        let mut mr = MessageReader { reader, last_update: current_time_secs(),msgs: vec![] };
        mr.read_messages().await?;
        Ok(mr)
    }

    pub async fn read_messages(&mut self) -> anyhow::Result<()>{
        let msgs = self.reader.fetch_raw_msgs().await;
        if msgs.len() > 0{
            self.last_update = current_time_secs();
        }
        for (_, p, _) in msgs{
            let packet = JsonPacket::from_streams_response(&p, &vec![], &None)?;
            self.msgs.push(packet.deserialize_public()?);
        }
        Ok(())
    }
    
    pub fn msgs(&self) -> &Vec<HashMap<String, Value>> {
        &self.msgs
    }

    pub fn last_updates_seconds_ago(&self) -> i64{
        current_time_secs() - self.last_update
    }
}

fn create_channel(mainnet: bool) -> ChannelWriter{
    ChannelWriter::builder()
        .node(&node_url(mainnet))
        .build()
}

fn create_reader(channel_id: &str, announce_id: &str, mainnet:bool) -> ChannelReader{
    ChannelReader::builder()
        .node(&node_url(mainnet))
        .build(channel_id, announce_id)
}

fn node_url(mainnet: bool) -> String{
    if mainnet{
        return "https://chrysalis-nodes.iota.cafe/".to_string();
    }
    "https://api.lb-0.testnet.chrysalis2.com".to_string()
}
