use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use crate::channels::{Category, create_channel, ChannelInfo};
use crate::utils::{current_time_secs, timestamp_to_date_string};

pub struct DailyChannel{
    category: Category,
    actor_id: String,
    channel: ChannelWriter,
    creation_timestamp: i64
}

impl DailyChannel{
    pub fn new(category: Category, actor_id: &str, mainnet: bool) -> Self {
        let creation_timestamp = current_time_secs();
        let channel = create_channel(mainnet);
        DailyChannel { category, actor_id: actor_id.to_lowercase(), channel, creation_timestamp}
    }

    pub async fn open(&mut self, state_psw: &str) -> anyhow::Result<ChannelInfo>{
        let info = self.channel.open_and_save(state_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }

    pub fn creation_date(&self) -> String{
        timestamp_to_date_string(self.creation_timestamp, false)
    }

    pub fn channel_info(&self) -> ChannelInfo{
        let info = self.channel.channel_address();
        ChannelInfo::new(info.0, info.1)
    }

    pub fn print_nested_channel_info(&self){
        let info = self.channel_info();
        println!("          Day {} = {}:{}", self.creation_date(), info.channel_id, info.announce_id);
    }
}

