use serde::{Serialize, Deserialize};
use iota_streams_lib::channels::{ChannelWriter, ChannelReader};

pub mod root_channel;
mod category_channel;
mod actor_channel;
mod daily_channel;
pub use category_channel::ActorChannelMsg as ActorChannelInfo;
pub use actor_channel::DailyChannelMsg as DailyChannelInfo;
use std::collections::HashMap;
use serde_json::Value;
use iota_streams_lib::payload::payload_serializers::RawPacket;

#[derive(Debug, Clone)]
pub enum Category{
    Trucks,
    Scales,
    BioCells
}

impl Category{
    pub fn is_trucks(&self) -> bool{
        match self{
            Category::Trucks => true,
            _ => false
        }
    }
    pub fn is_scales(&self) -> bool{
        match self{
            Category::Scales => true,
            _ => false
        }
    }
    pub fn is_biocells(&self) -> bool{
        match self{
            Category::BioCells => true,
            _ => false
        }
    }
    pub fn equals_to(&self, other: &Category) -> bool{
        match (self, other) {
            (Category::Trucks, Category::Trucks) => true,
            (Category::Scales, Category::Scales) => true,
            (Category::BioCells, Category::BioCells) => true,
            (_, _) => false
        }
    }

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
            "scales" => Some(Category::Scales),
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
        format!("https://streams-chrysalis-explorer.netlify.app/channel/{}:{}", self.channel_id, self.announce_id)
    }
}

pub struct MessageReader{
    reader: ChannelReader,
    msgs: Vec<HashMap<String, Value>>,
}

impl MessageReader{
    pub async fn new(channel_info: &ChannelInfo, mainnet: bool) -> anyhow::Result<Self> {
        let mut reader = create_reader(channel_info.channel_id(), channel_info.announce_id(), mainnet);
        reader.attach().await?;
        let mut mr = MessageReader { reader, msgs: vec![] };
        mr.read_messages().await?;
        Ok(mr)
    }

    pub async fn read_messages(&mut self) -> anyhow::Result<()>{
        let msgs = self.reader.fetch_raw_msgs().await;
        for (_, p, _) in msgs{
            let packet = RawPacket::from_streams_response(&p, &vec![], &None)?;
            self.msgs.push(packet.deserialize_public()?);
        }
        Ok(())
    }
    
    pub fn msgs(&self) -> &Vec<HashMap<String, Value>> {
        &self.msgs
    }
}

fn create_channel(mainnet: bool) -> ChannelWriter{
    if mainnet{
        return ChannelWriter::builder().node("https://chrysalis-nodes.iota.cafe/").build();
    }
    ChannelWriter::builder().build()
}

fn create_reader(channel_id: &str, announce_id: &str, mainnet:bool) -> ChannelReader{
    if mainnet{
        return ChannelReader::builder().node("https://chrysalis-nodes.iota.cafe/").build(channel_id, announce_id);
    }
    ChannelReader::builder().build(channel_id, announce_id)
}
