use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use crate::channels::daily_channel::DailyChannel;
use crate::channels::{Category, create_channel, ChannelInfo, create_reader};
use crate::utils::{current_time_secs, timestamp_to_date};
use chrono::Datelike;
use iota_streams_lib::payload::payload_serializers::{JsonPacketBuilder, JsonPacket};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DailyChannelMsg{
    address: ChannelInfo,
    category: String,
    actor_id: String,
    creation_timestamp: i64,
}

impl DailyChannelMsg{
    pub fn new(address: ChannelInfo, category: Category, actor_id: &str, creation_timestamp: i64) -> Self {
        DailyChannelMsg { address, category: category.to_string(), actor_id: actor_id.to_lowercase(), creation_timestamp }
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
    pub fn creation_timestamp(&self) -> i64 {
        self.creation_timestamp
    }
}

pub struct ActorChannel{
    category: Category,
    actor_id: String,
    channel: ChannelWriter,
    daily_channels: Vec<DailyChannel>,
    mainnet: bool,
}

impl ActorChannel{
    pub fn new(category: Category, actor_id: &str, mainnet: bool) -> Self {
        let channel = create_channel(mainnet);
        ActorChannel { category, actor_id: actor_id.to_lowercase(), channel, daily_channels: vec![], mainnet }
    }

    pub async fn import_from_tangle(channel_id: &str, announce_id: &str, state_psw: &str, category: Category, actor_id: &str, mainnet: bool) -> anyhow::Result<Self>{
        let node = if mainnet{
            Some("https://chrysalis-nodes.iota.cafe/")
        }else{
            None
        };
        let channel = ChannelWriter::import_from_tangle(channel_id, announce_id, state_psw, node, None).await?;
        let daily_info = ActorChannel::read_daily_channels_info(channel_id, announce_id, mainnet).await?;
        let mut daily_channels = vec![];
        for d in daily_info {
            let ch = DailyChannel::import_from_tangle(
                &d.address.channel_id,
                &d.address.announce_id,
                state_psw,
                category.clone(),
                actor_id,
                d.creation_timestamp(),
                mainnet
            ).await?;
            daily_channels.push(ch)
        }
        Ok( ActorChannel{category, actor_id: actor_id.to_lowercase(), channel, daily_channels, mainnet } )
    }

    pub async fn open(&mut self, channel_psw: &str) -> anyhow::Result<ChannelInfo> {
        let info = self.channel.open_and_save(channel_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }

    pub async fn create_daily_channel_in_date(&mut self, state_psw: &str, day: u16, month: u16, year: u16) -> anyhow::Result<ChannelInfo>{
        let date_string = format!("{:02}/{:02}/{}", day, month, year);
        let found = self.daily_channels.iter()
            .filter(|ch| { date_string == ch.creation_date() })
            .count();
        if found > 0{
            return Err(anyhow::Error::msg(format!("Daily channel already exist in date {}", date_string)));
        }

        let mut daily_channel = DailyChannel::new_in_date(
            self.category.clone(), self.actor_id(),
            day, month, year,
            self.mainnet
        )?;

        let timestamp = daily_channel.creation_timestamp();
        let info = daily_channel.open(state_psw).await?;
        self.daily_channels.push(daily_channel);
        self.publish_daily_channel(info.clone(), timestamp).await?;
        Ok(info)
    }

    pub async fn create_daily_channel(&mut self, state_psw: &str) -> anyhow::Result<ChannelInfo>{
        let date = timestamp_to_date(current_time_secs(), false);
        self.create_daily_channel_in_date(state_psw, date.day() as u16, date.month() as u16, date.year() as u16).await
    }

    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }

    pub fn channel_info(&self) -> ChannelInfo{
        let info = self.channel.channel_address();
        ChannelInfo::new(info.0, info.1)
    }

    pub fn print_nested_channel_info(&self){
        let info = self.channel_info();
        println!("      Actor {} = {}:{}", self.actor_id, info.channel_id, info.announce_id);

        self.daily_channels.iter().for_each(|ch| ch.print_nested_channel_info());
        println!();
    }
}

impl ActorChannel{
    async fn publish_daily_channel(&mut self, info: ChannelInfo, timestamp: i64) -> anyhow::Result<()>{
        let msg = DailyChannelMsg::new(info, self.category.clone(), self.actor_id(), timestamp);
        let packet = JsonPacketBuilder::new()
            .public(&msg)?
            .build();
        self.channel.send_signed_packet(&packet).await?;
        Ok(())
    }

    async fn read_daily_channels_info(channel_id: &str, announce_id: &str, mainnet: bool) -> anyhow::Result<Vec<DailyChannelMsg>>{
        let mut reader = create_reader(channel_id, announce_id, mainnet);
        reader.attach().await?;
        let msgs: Vec<(String, JsonPacket)> = reader.fetch_parsed_msgs(&None).await?;
        let mut daily_ch_info = vec![];
        for (_, m) in msgs {
            daily_ch_info.push(m.deserialize_public()?)
        }
        Ok(daily_ch_info)
    }
}

