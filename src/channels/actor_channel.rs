use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use crate::channels::daily_channel::DailyChannel;
use crate::channels::{Category, create_channel, ChannelInfo, create_reader};
use crate::utils::{current_time_secs, timestamp_to_date, timestamp_to_date_string};
use chrono::Datelike;
use iota_streams_lib::payload::payload_serializers::{JsonPacketBuilder, JsonPacket};
use serde::{Serialize, Deserialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DailyChannelMsg{
    address: ChannelInfo,
    category: String,
    actor_id: String,
    creation_timestamp: i64,
}

#[allow(dead_code)]
impl DailyChannelMsg{
    pub (crate) fn new(address: ChannelInfo, category: Category, actor_id: &str, creation_timestamp: i64) -> Self {
        DailyChannelMsg { address, category: category.to_string(), actor_id: actor_id.to_lowercase(), creation_timestamp }
    }
    pub (crate) fn address(&self) -> &ChannelInfo {
        &self.address
    }
    pub (crate) fn category(&self) -> &str {
        &self.category
    }
    pub (crate) fn actor_id(&self) -> &str {
        &self.actor_id
    }
    pub (crate) fn creation_timestamp(&self) -> i64 {
        self.creation_timestamp
    }
    pub (crate) fn creation_date(&self) -> String{
        timestamp_to_date_string(self.creation_timestamp, false)
    }
    pub (crate) fn print_nested_channel_info(&self){
        if self.category == String::from("biocells"){
            println!("        |--Day {} = {}:{}", self.creation_date(), self.address.channel_id, self.address.announce_id);
        }else{
            println!("|   |   |--Day {} = {}:{}", self.creation_date(), self.address.channel_id, self.address.announce_id);
        }
    }
}

pub (crate) struct ActorChannel{
    category: Category,
    actor_id: String,
    channel: ChannelWriter,
    daily_channels: Vec<DailyChannelMsg>,
    imported_channels: Vec<Rc<RefCell<DailyChannel>>>,
    mainnet: bool,
}

impl ActorChannel{
    pub (crate) fn new(category: Category, actor_id: &str, mainnet: bool) -> Self {
        let channel = create_channel(mainnet);
        ActorChannel { category, actor_id: actor_id.to_lowercase(), channel, daily_channels: vec![], imported_channels: vec![], mainnet }
    }

    pub (crate) async fn import_from_tangle(channel_id: &str, announce_id: &str, state_psw: &str, category: Category, actor_id: &str, mainnet: bool) -> anyhow::Result<Self>{
        let node = if mainnet{
            Some("https://chrysalis-nodes.iota.cafe/")
        }else{
            None
        };
        let channel = ChannelWriter::import_from_tangle(channel_id, announce_id, state_psw, node, None).await?;
        let daily_channels = ActorChannel::read_daily_channels_info(channel_id, announce_id, mainnet).await?;
        /*let mut daily_channels = vec![];
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
            daily_channels.push(Rc::new(RefCell::new(ch)));
        }*/
        Ok( ActorChannel{category, actor_id: actor_id.to_lowercase(), channel, daily_channels, imported_channels: vec![], mainnet } )
    }

    pub (crate) async fn open(&mut self, channel_psw: &str) -> anyhow::Result<ChannelInfo> {
        let info = self.channel.open_and_save(channel_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }

    pub (crate) async fn get_or_create_daily_channel_in_date(&mut self, state_psw: &str, day: u16, month: u16, year: u16) -> anyhow::Result<DailyChannelManager>{
        // Cerco se la data è presente all'interno dei daily channel msgs
        let date_string = format!("{:02}/{:02}/{}", day, month, year);
        let daily_ch_msg = self.daily_channels.iter()
            .find(|ch| { date_string == ch.creation_date() });

        let (ch, daily_ch_msg) = match daily_ch_msg {
            None => { // Se non è stata trovata la data corrispondente allora viene creato un nuovo channel
                let mut daily_channel = DailyChannel::new_in_date(
                    self.category.clone(), self.actor_id(), day, month, year, self.mainnet
                )?;
                let timestamp = daily_channel.creation_timestamp();
                let info = daily_channel.open(state_psw).await?;
                let daily_ch_msg = self.publish_daily_channel(info, timestamp).await?;
                self.daily_channels.push(daily_ch_msg);
                let cell = Rc::new(RefCell::new(daily_channel));
                self.imported_channels.push(cell.clone());
                return Ok(DailyChannelManager::new(cell));
            },
            Some(info) => { // Altrimenti si ricerca agli interno degli imported
                ( self.imported_channels.iter_mut()
                     .find(|ch| ch.borrow().creation_date() == date_string), info )
            }
        };


        let res = match ch{
            None => { // Se tra gli imported non è stato trovato si tenta un ripristino dal tangle
                let channel_id = daily_ch_msg.address.channel_id();
                let announce_id = daily_ch_msg.address.announce_id();
                DailyChannel::import_from_tangle(
                    channel_id, announce_id,
                    state_psw, self.category.clone(),
                    self.actor_id(),
                    daily_ch_msg.creation_timestamp(),
                    self.mainnet
                ).await
            },
            Some(ch) => return Ok(DailyChannelManager::new(ch.clone())) // Altrimenti si ritorna direttamente
        };

        match res{
            Ok(res) => {
                let cell = Rc::new(RefCell::new(res));
                self.imported_channels.push(cell.clone());
                Ok(DailyChannelManager::new(cell))
            } // Se c'è stato un errore durante il restore dal tangle probabilmente la password inserita sarà sbagliata
            Err(_) => Err(anyhow::Error::msg(format!("Impossible to get the channel in date {} because password is wrong", date_string)))
        }
    }

    #[allow(dead_code)]
    pub (crate) async fn get_or_create_daily_channel(&mut self, state_psw: &str) -> anyhow::Result<DailyChannelManager>{
        let date = timestamp_to_date(current_time_secs(), false);
        self.get_or_create_daily_channel_in_date(state_psw, date.day() as u16, date.month() as u16, date.year() as u16).await

    }

    pub (crate) fn actor_id(&self) -> &str {
        &self.actor_id
    }

    pub (crate) fn channel_info(&self) -> ChannelInfo{
        let info = self.channel.channel_address();
        ChannelInfo::new(info.0, info.1)
    }

    pub (crate) fn print_nested_channel_info(&self){
        let info = self.channel_info();
        if self.category.is_biocells(){
            println!("    |--Actor {} = {}:{}", self.actor_id, info.channel_id, info.announce_id);
        }else{
            println!("|   |--Actor {} = {}:{}", self.actor_id, info.channel_id, info.announce_id);
        }

        self.daily_channels.iter().for_each(|ch| ch.print_nested_channel_info());
    }
}

impl ActorChannel{
    async fn publish_daily_channel(&mut self, info: ChannelInfo, timestamp: i64) -> anyhow::Result<DailyChannelMsg>{
        let msg = DailyChannelMsg::new(info, self.category.clone(), self.actor_id(), timestamp);
        let packet = JsonPacketBuilder::new()
            .public(&msg)?
            .build();
        self.channel.send_signed_packet(&packet).await?;
        Ok(msg)
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

pub struct DailyChannelManager{
    daily_channel: Rc<RefCell<DailyChannel>>,
}

impl DailyChannelManager {
    fn new(daily_channel: Rc<RefCell<DailyChannel>>) -> Self {
        DailyChannelManager { daily_channel }
    }

    pub async fn send_raw_packet(&mut self, p_data: Vec<u8>, m_data: Vec<u8>, key_nonce: Option<([u8;32], [u8;24])>) -> anyhow::Result<String>{
        self.daily_channel.borrow_mut().send_raw_packet(p_data, m_data, key_nonce).await
    }

    pub fn creation_timestamp(&self) -> i64 {
        self.daily_channel.borrow().creation_timestamp()
    }

    pub fn creation_date(&self) -> String{
        self.daily_channel.borrow().creation_date()
    }

    pub fn channel_info(&self) -> ChannelInfo{
        self.daily_channel.borrow().channel_info()
    }
}
