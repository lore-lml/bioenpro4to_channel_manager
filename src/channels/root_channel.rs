use crate::channels::category_channel::CategoryChannel;
use crate::channels::{Category, create_channel, CategoryChannelsInfo, ChannelInfo};
use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use iota_streams_lib::payload::payload_serializers::JsonPacketBuilder;


pub struct RootChannel{
    root: ChannelWriter,
    truck_category: CategoryChannel,
    weighing_scale_category: CategoryChannel,
    biocell_category: CategoryChannel,
}


impl RootChannel{
    pub fn new(mainnet: bool) -> Self {
        let truck_category = CategoryChannel::new(Category::Trucks, mainnet);
        let weighing_scale_category = CategoryChannel::new(Category::Scales, mainnet);
        let biocell_category = CategoryChannel::new(Category::BioCells, mainnet);
        let root = create_channel(mainnet);
        RootChannel { root, truck_category, weighing_scale_category, biocell_category }
    }

    pub async fn open(&mut self, channel_psw: &str) -> anyhow::Result<ChannelInfo> {
        // Opening Channels Category Info
        self.truck_category.open(channel_psw).await?;
        self.weighing_scale_category.open(channel_psw).await?;
        self.biocell_category.open(channel_psw).await?;

        // Opening the root channel
        let root_info = self.root.open_and_save(channel_psw).await?;
        Ok(ChannelInfo::new(root_info.0, root_info.1))
    }

    pub async fn init_categories(&mut self) -> anyhow::Result<()>{
        let truck_info = self.truck_category.channel_info();
        let scale_info = self.weighing_scale_category.channel_info();
        let biocell_info = self.biocell_category.channel_info();

        //Creating MSG to send containing the info for every category channel
        let categories_info = CategoryChannelsInfo::new(truck_info, scale_info, biocell_info);
        let packet = JsonPacketBuilder::new()
            .public(&categories_info)?
            .build();
        self.root.send_signed_packet(&packet).await?;
        Ok(())
    }

    pub async fn create_daily_actor_channel(&mut self, category: Category, actor_id: &str, state_psw: &str,
                                                day: u16, month: u16, year: u16) -> anyhow::Result<ChannelInfo>{

        match category{
            Category::Trucks => self.truck_category.create_daily_actor_channel(actor_id, state_psw, day, month, year).await,
            Category::Scales => self.weighing_scale_category.create_daily_actor_channel(actor_id, state_psw, day, month, year).await,
            Category::BioCells => self.biocell_category.create_daily_actor_channel(actor_id, state_psw, day, month, year).await
        }

    }

    pub fn channel_info(&self) -> ChannelInfo{
        let info = self.root.channel_address();
        ChannelInfo::new(info.0, info.1)
    }

    pub fn print_nested_channel_info(&self){
        let info = self.channel_info();
        println!("Root = {}:{}", info.channel_id, info.announce_id);

        self.truck_category.print_nested_channel_info();
        self.weighing_scale_category.print_nested_channel_info();
        self.biocell_category.print_nested_channel_info();
        println!();
    }
}
