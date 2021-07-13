mod messages;
use bioenpro4to_channel_manager::channels::root_channel::RootChannel;
use bioenpro4to_channel_manager::channels::{Category, ChannelInfo};
use bioenpro4to_channel_manager::utils::{create_encryption_key, create_encryption_nonce, current_time_secs};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message{
    msg: String,
    timestamp: i64,
}
impl Message{
    fn new(msg: &str) -> Self {
        Message { msg: msg.to_string(), timestamp: current_time_secs() }
    }

    fn to_json(&self) -> anyhow::Result<Vec<u8>>{
        Ok(serde_json::to_string(self)?.as_bytes().to_vec())
    }
}

async fn test_create_nested_channels(state_psw: &str, mainnet: bool, key_nonce: Option<([u8; 32],[u8; 24])>) -> anyhow::Result<ChannelInfo>{
    let mut root = RootChannel::new(mainnet);
    let info = root.open(state_psw).await?;
    let state_psw = "ciaone";
    root.new_daily_actor_channel(Category::Trucks, "XASD", state_psw, 25, 5, 2021).await?;
    root.new_daily_actor_channel(Category::Trucks, "XASD", state_psw, 26, 5, 2021).await?;
    root.new_daily_actor_channel(Category::Trucks, "XASD2", state_psw, 25, 5, 2021).await?;
    let mut scale_ch = root.new_daily_actor_channel(Category::Scales, "SCALE1", state_psw, 28, 5, 2021).await?;

    let mut daily_ch = root.get_daily_actor_channel(Category::Trucks, "XASD", state_psw, 25, 5, 2021).await?;
    let public = Message::new("PUBLIC MESSAGE");
    let private = Message::new("PRIVATE MESSAGE");
    daily_ch.send_raw_packet(public.to_json()?, private.to_json()?, key_nonce).await?;
    scale_ch.send_raw_packet(public.to_json()?, private.to_json()?, key_nonce).await?;
    root.print_nested_channel_info();
    Ok(info)
}

async fn test_restore_nested_channels(info: ChannelInfo, state_psw: &str, mainnet: bool, key_nonce: Option<([u8; 32],[u8; 24])>) -> anyhow::Result<()>{
    let channel_id = info.channel_id();
    let announce_id = info.announce_id();
    let mut root = RootChannel::import_from_tangle(
        channel_id,
        announce_id,
        state_psw,
        mainnet
    ).await?;
    let state_psw = "ciaone";
    root.new_daily_actor_channel(Category::Trucks, "XASD", state_psw, 27, 5, 2021).await?;
    let mut daily_ch = root.get_daily_actor_channel(Category::Trucks, "XASD", state_psw, 25, 5, 2021).await?;
    let mut biocell_ch = root.new_daily_actor_channel(Category::BioCells, "BIO1", state_psw, 30, 5, 2021).await?;
    let public = Message::new("PUBLIC MESSAGE");
    let private = Message::new("PRIVATE MESSAGE");
    daily_ch.send_raw_packet(public.to_json()?, private.to_json()?, key_nonce).await?;
    biocell_ch.send_raw_packet(public.to_json()?, private.to_json()?, key_nonce).await?;
    root.print_nested_channel_info();
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let key = create_encryption_key("This is a secret key");
    let nonce = create_encryption_nonce("This is a secret nonce");
    let key_nonce = Some((key, nonce));
    let state_psw = "psw";
    let mainnet = false;
    let info = test_create_nested_channels(state_psw, mainnet, key_nonce).await?;
    test_restore_nested_channels(info, state_psw, mainnet, key_nonce).await?;
    Ok(())
}
