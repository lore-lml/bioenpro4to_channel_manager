use crate::channels::{Category, create_channel, ChannelInfo, create_reader};
use crate::channels::actor_channel::{ActorChannel, DailyChannelManager, DailyChannelMsg};
use serde::{Serialize, Deserialize};
use iota_streams_lib::payload::payload_serializers::{JsonPacketBuilder, JsonPacket};
use iota_streams_lib::channels::ChannelWriter;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActorChannelMsg{
    address: ChannelInfo,
    category: String,
    actor_id: String,
}

#[allow(dead_code)]
impl ActorChannelMsg{
    pub fn new(address: ChannelInfo, category: Category, actor_id: &str) -> Self {
        ActorChannelMsg { address, category: category.to_string(), actor_id: actor_id.to_lowercase() }
    }
    pub fn address(&self) -> &ChannelInfo {
        &self.address
    }
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }
}

pub (crate) struct CategoryChannel{
    category: Category,
    channel: ChannelWriter,
    actors: Vec<ActorChannel>,
    mainnet: bool
}

#[allow(dead_code)]
impl CategoryChannel {
    pub (crate) fn new(category: Category, mainnet: bool) -> Self {
        let channel = create_channel(mainnet);
        CategoryChannel { category, channel, actors: vec![], mainnet }
    }

    pub (crate) async fn import_from_tangle(channel_id: &str, announce_id: &str, state_psw: &str, category: Category, mainnet: bool) -> anyhow::Result<Self>{
        let node = if mainnet{
            Some("https://chrysalis-nodes.iota.cafe/")
        }else{
            None
        };
        let channel = ChannelWriter::import_from_tangle(channel_id, announce_id, state_psw, node, None).await?;
        let actors_info = CategoryChannel::read_actors_channels_info(channel_id, announce_id, mainnet).await?;
        let mut actors = vec![];
        for a in actors_info {
            let ch = ActorChannel::import_from_tangle(
                &a.address.channel_id,
                &a.address.announce_id,
                state_psw,
                category.clone(),
                a.actor_id(),
                mainnet).await?;
            actors.push(ch);
        }
        Ok( CategoryChannel{ category, channel, actors, mainnet } )
    }

    pub (crate) async fn open(&mut self, channel_psw: &str) -> anyhow::Result<ChannelInfo> {
        let info = self.channel.open_and_save(channel_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }

    pub (crate) fn category(&self) -> &Category {
        &self.category
    }

    pub (crate) async fn new_daily_actor_channel(&mut self, actor_id: &str, root_psw: &str, state_psw: &str,
                                                   day: u16, month: u16, year: u16) -> anyhow::Result<DailyChannelManager>{
        let exist = self.actors.iter().any(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase());
        if !exist{
            self.create_actor_channel(actor_id, root_psw).await?;
        }

        self.actors.iter_mut()
            .find(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase()).unwrap()
            .new_daily_actor_channel(state_psw, day, month, year).await
    }

    pub (crate) async fn get_daily_actor_channel(&mut self, actor_id: &str, state_psw: &str,
                                         day: u16, month: u16, year: u16) -> anyhow::Result<DailyChannelManager>{

        let exist = self.actors.iter().any(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase());
        if !exist{
            return Err(anyhow::Error::msg(format!("Actor {} doesn't exist yet", actor_id)));
        }

        self.actors.iter_mut()
            .find(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase()).unwrap()
            .get_daily_channel_in_date(state_psw, day, month, year).await
    }

    pub (crate) async fn serialize_daily_actor_channel(&mut self, actor_id: &str, state_psw: &str,
                                                       day: u16, month: u16, year: u16) -> anyhow::Result<String>{
        let exist = self.actors.iter().any(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase());
        if !exist{
            return Err(anyhow::Error::msg(format!("Actor {} doesn't exist yet", actor_id)));
        }

        self.actors.iter_mut()
            .find(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase()).unwrap()
            .serialize_daily_channel(state_psw, day, month, year).await
    }

    pub (crate) fn channel_info(&self) -> ChannelInfo{
        let info = self.channel.channel_address();
        ChannelInfo::new(info.0, info.1)
    }

    pub (crate) fn print_nested_channel_info(&self){
        let info = self.channel_info();
        let category = match self.category{
            Category::Trucks => "Trucks",
            Category::Scales => "Scales",
            Category::BioCells => "BioCells"
        };

        println!("|--{} = https://streams-chrysalis-explorer.netlify.app/channel/{}:{}", category, info.channel_id, info.announce_id);
        self.actors.iter().for_each(|a| a.print_nested_channel_info());
    }
}

impl CategoryChannel{
    async fn create_actor_channel(&mut self, actor_id: &str, state_psw: &str) -> anyhow::Result<()>{
        let found = self.actors.iter()
            .filter(|a| {actor_id.to_lowercase() == a.actor_id().to_string()})
            .count();
        if found > 0{
            return Err(anyhow::Error::msg("Actor channel with this id already exist"));
        }
        let mut actor_channel = ActorChannel::new(self.category.clone(), actor_id, self.mainnet);
        let info = actor_channel.open(state_psw).await?;
        self.actors.push(actor_channel);

        self.publish_actor_channel(info, actor_id).await?;
        Ok(())
    }

    async fn publish_actor_channel(&mut self, info: ChannelInfo, actor_id: &str) -> anyhow::Result<()>{
        let msg = ActorChannelMsg::new(info, self.category.clone(), actor_id);
        let packet = JsonPacketBuilder::new()
            .public(&msg)?
            .build();
        self.channel.send_signed_packet(&packet).await?;
        Ok(())
    }

    async fn read_actors_channels_info(channel_id: &str, announce_id: &str, mainnet: bool) -> anyhow::Result<Vec<ActorChannelMsg>>{
        let mut reader = create_reader(channel_id, announce_id, mainnet);
        reader.attach().await?;
        let msgs: Vec<(String, JsonPacket)> = reader.fetch_parsed_msgs(&None).await?;
        let mut actors = vec![];
        for (_, m) in msgs {
            actors.push(m.deserialize_public()?);
        }
        Ok(actors)
    }
}

// Read APIs
impl CategoryChannel {
    pub fn actors_info(&self) -> Vec<ActorChannelMsg>{
        self.actors.iter().map(|a| {
            ActorChannelMsg::new(a.channel_info(), self.category.clone(), a.actor_id())
        })
            .collect()
    }

    pub fn channels_of_actor(&self, actor_id: &str) -> Vec<DailyChannelMsg>{
        self.actors.iter()
            .find(|a| a.actor_id().to_lowercase() == actor_id.to_lowercase())
            .map_or(vec![], |a| a.daily_channels_info())
    }
}
