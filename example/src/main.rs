use bioenpro4to_channel_manager::channels::root_channel::RootChannel;
use bioenpro4to_channel_manager::channels::{Category, ChannelInfo};

async fn test_create_nested_channels(state_psw: &str, mainnet: bool) -> anyhow::Result<ChannelInfo>{
    let mut root = RootChannel::new(mainnet);
    let info = root.open(state_psw).await?;
    root.init_categories().await?;
    root.create_daily_actor_channel(Category::Trucks, "XASD", state_psw, 25,5,2021).await?;
    root.create_daily_actor_channel(Category::Trucks, "XASD", state_psw, 26,5,2021).await?;
    root.create_daily_actor_channel(Category::Trucks, "XASD2", state_psw, 25,5,2021).await?;
    root.print_nested_channel_info();
    Ok(info)
}

async fn test_restore_nested_channels(info: ChannelInfo, state_psw: &str, mainnet: bool) -> anyhow::Result<()>{
    let channel_id = &info.channel_id;
    let announce_id = &info.announce_id;
    let mut root = RootChannel::import_from_tangle(
        channel_id,
        announce_id,
        state_psw,
        mainnet
    ).await?;
    root.create_daily_actor_channel(Category::Trucks, "XASD", state_psw, 27,5,2021).await?;
    root.print_nested_channel_info();
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let state_psw = "psw";
    let mainnet = false;
    let info = test_create_nested_channels(state_psw, mainnet).await?;
    test_restore_nested_channels(info, state_psw, mainnet).await?;
    Ok(())
}
