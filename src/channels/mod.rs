use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use iota_streams_lib::channel::builders::channel_builders::{ChannelWriterBuilder, ChannelReaderBuilder};
use serde::{Serialize, Deserialize};
use iota_streams_lib::channel::tangle_channel_reader::ChannelReader;

pub mod root_channel;
mod category_channel;
pub mod actor_channel;
mod daily_channel;

#[derive(Clone)]
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

fn create_channel(mainnet: bool) -> ChannelWriter{
    if mainnet{
        return ChannelWriterBuilder::new().node("https://chrysalis-nodes.iota.cafe/").build();
    }
    ChannelWriterBuilder::new().build()
}

fn create_reader(channel_id: &str, announce_id: &str, mainnet:bool) -> ChannelReader{
    if mainnet{
        return ChannelReaderBuilder::new().node("https://chrysalis-nodes.iota.cafe/").build(channel_id, announce_id);
    }
    ChannelReaderBuilder::new().build(channel_id, announce_id)
}
