use crate::channels::{Category, create_channel, ChannelInfo};
use crate::utils::{current_time_secs, timestamp_to_date_string};
use chrono::NaiveDate;
use iota_streams_lib::channels::ChannelWriter;

#[allow(dead_code)]
pub struct DailyChannel{
    category: Category,
    actor_id: String,
    channel: ChannelWriter,
    creation_timestamp: i64
}

impl DailyChannel{
    pub (crate) fn new(category: Category, actor_id: &str, mainnet: bool) -> Self {
        let creation_timestamp = current_time_secs();
        let channel = create_channel(mainnet);
        DailyChannel { category, actor_id: actor_id.to_lowercase(), channel, creation_timestamp}
    }

    pub (crate) fn new_in_date(category: Category, actor_id: &str, day: u16, month: u16, year: u16, mainnet: bool) -> anyhow::Result<Self>{
        let mut ch = DailyChannel::new(category, actor_id, mainnet);
        let date = match NaiveDate::from_ymd(year as i32, month as u32, day as u32)
            .and_hms_opt(0, 0, 0){
            None => return Err(anyhow::Error::msg("Invalid date")),
            Some(date) => date
        };
        ch.creation_timestamp = date.timestamp();
        Ok(ch)
    }

    pub (crate) async fn import_from_tangle(channel_id: &str, announce_id: &str, state_psw: &str, category: Category,
                                    actor_id: &str, creation_timestamp: i64, mainnet: bool) -> anyhow::Result<Self>{
        let node = if mainnet{
            Some("https://chrysalis-nodes.iota.cafe/")
        }else{
            None
        };
        let channel = ChannelWriter::import_from_tangle(channel_id, announce_id, state_psw, node, None).await?;
        Ok(DailyChannel{ category, actor_id: actor_id.to_lowercase(), channel, creation_timestamp })
    }

    pub (crate) async fn open(&mut self, state_psw: &str) -> anyhow::Result<ChannelInfo>{
        let info = self.channel.open_and_save(state_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }
}

impl DailyChannel{
    pub async fn send_raw_packet(&mut self, p_data: Vec<u8>, m_data: Vec<u8>, key_nonce: Option<([u8;32], [u8;24])>) -> anyhow::Result<String>{
        self.channel.send_signed_raw_data(p_data, m_data, key_nonce).await
    }

    pub fn creation_timestamp(&self) -> i64 {
        self.creation_timestamp
    }

    pub fn creation_date(&self) -> String{
        timestamp_to_date_string(self.creation_timestamp, false)
    }

    pub fn channel_info(&self) -> ChannelInfo{
        let info = self.channel.channel_address();
        ChannelInfo::new(info.0, info.1)
    }
}

