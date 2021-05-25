use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use iota_streams_lib::channel::builders::channel_builders::ChannelWriterBuilder;
use serde::{Serialize, Deserialize};

pub mod daily_channel;
pub mod root_channel;
pub mod category_channel;
pub mod actor_channel;

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
pub struct CategoryChannelsInfo{
    pub trucks: ChannelInfo,
    pub weighing_scale: ChannelInfo,
    pub biocell: ChannelInfo,
}

impl CategoryChannelsInfo{
    pub fn new(trucks: ChannelInfo, weighing_scale: ChannelInfo, biocell: ChannelInfo) -> Self{
        CategoryChannelsInfo{ trucks, weighing_scale, biocell }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelInfo{
    pub channel_id: String,
    pub announce_id: String,
}

impl ChannelInfo{
    pub fn new(channel_id: String, announce_id: String) -> Self {
        ChannelInfo { channel_id, announce_id }
    }
}

pub fn create_channel(mainnet: bool) -> ChannelWriter{
    if mainnet{
        return ChannelWriterBuilder::new().node("https://chrysalis-nodes.iota.cafe/").build();
    }
    ChannelWriterBuilder::new().build()
}
