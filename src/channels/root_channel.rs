use crate::channels::category_channel::{CategoryChannel, ActorChannelMsg};
use crate::channels::{Category, create_channel, ChannelInfo, create_reader};
use iota_streams_lib::payload::payload_serializers::{JsonPacketBuilder, JsonPacket};
use serde::{Serialize, Deserialize};
use crate::channels::actor_channel::{DailyChannelManager, DailyChannelMsg};
use iota_streams_lib::channels::ChannelWriter;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryChannelsInfo{
    pub trucks: ChannelInfo,
    pub weighing_scales: ChannelInfo,
    pub biocells: ChannelInfo,
}

impl CategoryChannelsInfo{
    pub fn new(trucks: ChannelInfo, weighing_scale: ChannelInfo, biocell: ChannelInfo) -> Self{
        CategoryChannelsInfo{ trucks, weighing_scales: weighing_scale, biocells: biocell }
    }
}

pub struct RootChannel{
    root: ChannelWriter,
    categories: Vec<(Arc<Mutex<CategoryChannel>>, Category)>,
    psw: String,
    mainnet: bool
}


impl RootChannel{
    //
    // Build the Root Channel of the nested channel architecture of BioEnPro4To project
    //
    pub fn new(mainnet: bool) -> Self {
        let truck_category = (Arc::new(Mutex::new(CategoryChannel::new(Category::Trucks, mainnet))), Category::Trucks);
        let weighing_scale_category = (Arc::new(Mutex::new(CategoryChannel::new(Category::Scales, mainnet))), Category::Scales);
        let biocell_category = (Arc::new(Mutex::new(CategoryChannel::new(Category::BioCells, mainnet))), Category::BioCells);
        let root = create_channel(mainnet);
        RootChannel { root, categories: vec![truck_category, weighing_scale_category, biocell_category], psw: String::default(), mainnet }
    }

    //
    // Restore the entire nested architecture giving the address of the root channel and the password previously used for the encryption of the state
    //
    pub async fn import_from_tangle(channel_id: &str, announce_id: &str, state_psw: &str, mainnet: bool) -> anyhow::Result<Self>{
        let node = if mainnet{
            Some("https://chrysalis-nodes.iota.cafe/")
        }else{
            None
        };
        println!("Importing tree");
        let root = ChannelWriter::import_from_tangle(
            channel_id,
            announce_id,
            state_psw,
            node,
            None
        ).await?;
        println!("  Root imported");

        let categories_info = RootChannel::read_categories_channels_info(channel_id, announce_id, mainnet).await?;
        let categories = RootChannel::import_categories(categories_info, state_psw, mainnet).await?;

        Ok(RootChannel{
            root,
            categories: vec![
                (Arc::new(Mutex::new(categories.0)), Category::Trucks),
                (Arc::new(Mutex::new(categories.1)), Category::Scales),
                (Arc::new(Mutex::new(categories.2)), Category::BioCells)
            ],
            psw: state_psw.to_string(),
            mainnet
        })
    }

    //
    // Initialize and opens the first two layers of the nested architecture
    //
    pub async fn open(&mut self, channel_psw: &str) -> anyhow::Result<ChannelInfo> {
        // Opening Channels Category Info
        println!("Initializing channels...");
        self.categories[0].0.lock().unwrap().open(channel_psw).await?;
        println!("  Trucks tree initialized");
        self.categories[1].0.lock().unwrap().open(channel_psw).await?;
        println!("  Scales tree initialized");
        self.categories[2].0.lock().unwrap().open(channel_psw).await?;
        println!("  Biocells tree initialized");
        // Opening the root channel
        let root_info = self.root.open_and_save(channel_psw).await?;
        self.init_categories().await?;
        self.psw = channel_psw.to_string();
        Ok(ChannelInfo::new(root_info.0, root_info.1))
    }



    //
    // Create the daily channel for a given actor of a certain category for the specified date
    //

    pub async fn new_daily_actor_channel(&mut self, category: Category, actor_id: &str, state_psw: &str,
                                         day: u16, month: u16, year: u16) -> anyhow::Result<DailyChannelManager>{
        println!("Trying creating daily channel: ({}, {}, {:02}/{:02}/{})", category.to_string(), actor_id, day, month, year);
        let category = &self.categories.iter_mut().find(|cat| category == cat.1).unwrap().0;
        let res = category.lock().unwrap().new_daily_actor_channel(actor_id, &self.psw, state_psw, day, month, year).await;
        match res{
            Ok(res) => {
                println!("  Creation Complete");
                Ok(res)
            },
            Err(err) => {
                eprintln!("  {}", err.to_string());
                Err(err)
            }
        }
    }

    pub async fn get_daily_actor_channel(&mut self, category: Category, actor_id: &str, state_psw: &str,
                                         day: u16, month: u16, year: u16) -> anyhow::Result<DailyChannelManager>{
        println!("Getting daily channel: ({}, {}, {:02}/{:02}/{})", category.to_string(), actor_id, day, month, year);
        let category = &self.categories.iter_mut().find(|cat| category == cat.1).unwrap().0;
        let res = category.lock().unwrap().get_daily_actor_channel(actor_id, state_psw, day, month, year).await;
        match res{
            Ok(res) => {
                println!("  Getting Complete");
                Ok(res)
            },
            Err(err) => {
                eprintln!("{}", err.to_string());
                Err(err)
            }
        }
    }

    pub async fn serialize_daily_actor_channel(&mut self, category: Category, actor_id: &str, state_psw: &str,
                                               day: u16, month: u16, year: u16) -> anyhow::Result<String>{
        println!("Serializing daily channel: ({}, {}, {:02}/{:02}/{})", category.to_string(), actor_id, day, month, year);
        let category = &self.categories.iter_mut().find(|cat| category == cat.1).unwrap().0;
        let res = category.lock().unwrap().serialize_daily_actor_channel(actor_id, state_psw, day, month, year).await;
        println!("  Serializing complete");
        res
    }

    //
    // Returns the channel info of the root channel
    //
    pub fn channel_info(&self) -> ChannelInfo{
        let info = self.root.channel_address();
        ChannelInfo::new(info.0, info.1)
    }

    //
    // Tells if the root channel is attached to the mainnet or not
    //
    pub fn is_mainnet(&self) -> bool{
        self.mainnet
    }

    //
    // Print all the architecture in a hierarchical view
    //
    pub fn print_nested_channel_info(&self){
        let info = self.channel_info();
        println!("\nRoot = https://streams-chrysalis-explorer.netlify.app/channel/{}:{}?mainnet={}", info.channel_id, info.announce_id, self.mainnet);

        self.categories[0].0.lock().unwrap().print_nested_channel_info();
        self.categories[1].0.lock().unwrap().print_nested_channel_info();
        self.categories[2].0.lock().unwrap().print_nested_channel_info();
        println!();
    }
}

impl RootChannel {
    async fn init_categories(&mut self) -> anyhow::Result<()>{
        println!("Initializing tree messages...");
        let truck_info = self.categories[0].0.lock().unwrap().channel_info();
        let scale_info = self.categories[1].0.lock().unwrap().channel_info();
        let biocell_info = self.categories[2].0.lock().unwrap().channel_info();

        //Creating MSG to send containing the info for every category channel
        let categories_info = CategoryChannelsInfo::new(truck_info, scale_info, biocell_info);
        let packet = JsonPacketBuilder::new()
            .public(&categories_info)?
            .build();
        self.root.send_signed_packet(&packet).await?;
        println!("  Initial messages sent");
        Ok(())
    }

    async fn read_categories_channels_info(channel_id: &str, announce_id: &str, mainnet: bool) -> anyhow::Result<CategoryChannelsInfo>{
        let mut reader = create_reader(channel_id, announce_id, mainnet);
        reader.attach().await?;
        let mut msgs: Vec<(String, JsonPacket)> = reader.fetch_parsed_msgs(&None).await?;
        match msgs.pop(){
            None => Err(anyhow::Error::msg("Error during parsing categories channels")),
            Some((_, packet)) => packet.deserialize_public()
        }
    }

    async fn import_categories(categories_info: CategoryChannelsInfo, state_psw: &str, mainnet: bool) -> anyhow::Result<(CategoryChannel, CategoryChannel, CategoryChannel)>{
        println!("Importing categories...");
        let truck_category = CategoryChannel::import_from_tangle(
            &categories_info.trucks.channel_id,
            &categories_info.trucks.announce_id,
            state_psw,
            Category::Trucks,
            mainnet
        ).await?;
        println!("  Trucks imported");
        let weighing_scale_category = CategoryChannel::import_from_tangle(
            &categories_info.weighing_scales.channel_id,
            &categories_info.weighing_scales.announce_id,
            state_psw,
            Category::Scales,
            mainnet
        ).await?;
        println!("  Scales imported");
        let biocell_category = CategoryChannel::import_from_tangle(
            &categories_info.biocells.channel_id,
            &categories_info.biocells.announce_id,
            state_psw,
            Category::BioCells,
            mainnet
        ).await?;
        println!("  Biocells imported");
        Ok((truck_category, weighing_scale_category, biocell_category))
    }
}


// Read APIs
impl RootChannel{
    pub fn actors_of_category(&self, category: Category) -> Vec<ActorChannelMsg>{
        let cat = &self.categories.iter().find(|cat| category == cat.1).unwrap().0;
        cat.lock().unwrap().actors_info()
    }

    pub fn channels_of_actor(&self, category: Category, actor_id: &str) -> Vec<DailyChannelMsg>{
        let cat = &self.categories.iter().find(|cat| category == cat.1).unwrap().0;
        cat.lock().unwrap().channels_of_actor(actor_id)
    }

}
