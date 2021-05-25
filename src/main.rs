use bioenpro4to_channel_manager::channels::root_channel::RootChannel;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let state_psw = "psw";
    let mut root = RootChannel::new(false);
    root.open(state_psw).await?;
    root.init_categories().await?;
    root.print_nested_channel_info();
    Ok(())
}
