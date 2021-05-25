use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use crate::channels::daily_channel::DailyChannel;
use crate::channels::{Category, create_channel, ChannelInfo};
use crate::utils::{current_time_secs, timestamp_to_date};
use chrono::{NaiveDate, Datelike};

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

    pub async fn open(&mut self, channel_psw: &str) -> anyhow::Result<ChannelInfo> {
        let info = self.channel.open_and_save(channel_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }

    pub async fn create_daily_channel_in_date(&mut self, state_psw: &str, day: u16, month: u16, year: u16) -> anyhow::Result<ChannelInfo>{
        match NaiveDate::from_ymd(year as i32, month as u32, day as u32)
            .and_hms_opt(0, 0, 0){
            None => return Err(anyhow::Error::msg("Invalid date")),
            _ => {}
        };

        let date_string = format!("{:02}/{:02}/{}", day, month, year);
        let found = self.daily_channels.iter()
            .filter(|ch| { date_string == ch.creation_date() })
            .count();
        if found > 0{
            return Err(anyhow::Error::msg(format!("Daily channel already exist in date {}", date_string)));
        }

        let mut daily_channel = DailyChannel::new(self.category.clone(), self.actor_id(), self.mainnet);
        let info = daily_channel.open(state_psw).await?;
        self.daily_channels.push(daily_channel);
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

