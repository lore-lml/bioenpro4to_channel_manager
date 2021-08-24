use crate::channels::{Category, create_channel, ChannelInfo, node_url};
use crate::utils::{current_time_secs, timestamp_to_date_string, hash_string};
use chrono::NaiveDate;
use iota_streams_lib::channels::ChannelWriter;
use serde::{Serialize, Deserialize};
use aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::XChaCha20Poly1305;
use base64::{encode_config, URL_SAFE_NO_PAD, decode_config};

#[allow(dead_code)]
pub struct DailyChannel{
    category: Category,
    actor_id: String,
    channel: ChannelWriter,
    creation_timestamp: i64,
    mainnet: bool
}

impl DailyChannel{
    pub (crate) fn new(category: Category, actor_id: &str, mainnet: bool) -> Self {
        let creation_timestamp = current_time_secs();
        let channel = create_channel(mainnet);
        DailyChannel { category, actor_id: actor_id.to_lowercase(), channel, creation_timestamp, mainnet}
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
        Ok(DailyChannel{ category, actor_id: actor_id.to_lowercase(), channel, creation_timestamp, mainnet})
    }

    pub (crate) async fn open(&mut self, state_psw: &str) -> anyhow::Result<ChannelInfo>{
        let info = self.channel.open_and_save(state_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }

    pub (crate) fn export_to_base64(&self, state_psw: &str) -> anyhow::Result<String>{
        let state = DailyChannelState::new(state_psw, &self)?;
        state.encrypt()
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

    pub async fn import_from_base64(state: &str, state_psw: &str) -> anyhow::Result<Self>{
        println!("Trying to import daily channel from base64...");
        let state = DailyChannelState::decrypt(state, state_psw)?;
        let ch = state.to_daily_channel().await?;
        println!("  ...Daily Channel Import complete");
        Ok(ch)
    }
}

#[derive(Serialize, Deserialize)]
struct DailyChannelState{
    channel_state: Vec<u8>,
    category: Category,
    actor_id: String,
    creation_timestamp: i64,
    state_psw: String,
    mainnet: bool
}

impl DailyChannelState {
    fn new(state_psw: &str, channel: &DailyChannel) -> anyhow::Result<Self>{
        let channel_state = channel.channel.export_to_bytes(state_psw)?;
        let category = channel.category.clone();
        let actor_id = channel.actor_id.clone();
        let creation_timestamp = channel.creation_timestamp;
        let state_psw = state_psw.to_string();
        let mainnet = channel.mainnet;
        Ok(DailyChannelState{channel_state, category, actor_id, creation_timestamp, state_psw, mainnet})
    }

    pub fn encrypt(&self) -> anyhow::Result<String>{
        let psw = &self.state_psw;
        let bytes = bincode::serialize(&self)?;

        let (key, nonce) = key_nonce(psw);
        let key = GenericArray::from_slice(&key[..]);
        let nonce = GenericArray::from_slice(&nonce[..]);

        let chacha = XChaCha20Poly1305::new(key);
        let enc = chacha.encrypt(nonce, bytes.as_ref())
            .map_err(|_| anyhow::Error::msg("Error during state encryption"))?;

        let base64 = encode_config(&enc, URL_SAFE_NO_PAD);
        Ok(base64)
    }

    pub fn decrypt(base64: &str, psw: &str) -> anyhow::Result<Self>{
        let bytes = decode_config(&base64.as_bytes().to_vec(), URL_SAFE_NO_PAD)?;

        let (key, nonce) = key_nonce(psw);
        let key = GenericArray::from_slice(&key[..]);
        let nonce = GenericArray::from_slice(&nonce[..]);

        let chacha = XChaCha20Poly1305::new(key);
        let dec = chacha.decrypt(nonce, bytes.as_ref())
            .map_err(|_| anyhow::Error::msg("Error during state decryption"))?;

        let ch_state: DailyChannelState = bincode::deserialize(&dec)?;
        assert_eq!(psw.to_string(), ch_state.state_psw);
        Ok(ch_state)
    }

    pub async fn to_daily_channel(&self) -> anyhow::Result<DailyChannel>{
        let writer = ChannelWriter::import_from_bytes(
            &self.channel_state,
            &self.state_psw,
            Some(&node_url(self.mainnet)),
            None
        ).await?;
        let daily_ch = DailyChannel{
            category: self.category.clone(),
            actor_id: self.actor_id.clone(),
            channel: writer,
            creation_timestamp: self.creation_timestamp,
            mainnet: self.mainnet
        };
        Ok(daily_ch)
    }
}

fn key_nonce(psw: &str) -> (Vec<u8>, Vec<u8>) {
    let key_hash = &hash_string(psw)[..32];
    let nonce_hash = &hash_string(key_hash)[..24];
    let key = key_hash.as_bytes();
    let nonce = nonce_hash.as_bytes();
    (key.to_vec(), nonce.to_vec())
}
