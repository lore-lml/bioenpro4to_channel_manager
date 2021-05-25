use bioenpro4to_channel_manager::channels::root_channel::RootChannel;
use bioenpro4to_channel_manager::channels::Category;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let state_psw = "psw";
    let mut root = RootChannel::new(false);
    root.open(state_psw).await?;
    root.init_categories().await?;
    root.create_daily_actor_channel(Category::Trucks, "XASD", state_psw, 25,5,2021).await?;
    root.create_daily_actor_channel(Category::Trucks, "XASD", state_psw, 26,5,2021).await?;
    root.create_daily_actor_channel(Category::Trucks, "XASD2", state_psw, 25,5,2021).await?;
    root.print_nested_channel_info();
    Ok(())
}
