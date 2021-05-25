use crate::channels::{Category, create_channel, ChannelInfo};
use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use crate::channels::actor_channel::ActorChannel;

pub struct CategoryChannel{
    category: Category,
    channel: ChannelWriter,
    actors: Vec<ActorChannel>,
    mainnet: bool
}

impl CategoryChannel {
    pub fn new(category: Category, mainnet: bool) -> Self {
        let channel = create_channel(mainnet);
        CategoryChannel { category, channel, actors: vec![], mainnet }
    }

    pub async fn open(&mut self, channel_psw: &str) -> anyhow::Result<ChannelInfo> {
        let info = self.channel.open_and_save(channel_psw).await?;
        Ok(ChannelInfo::new(info.0, info.1))
    }



    pub fn category(&self) -> &Category {
        &self.category
    }

    pub async fn create_daily_actor_channel(&mut self, actor_id: &str, state_psw: &str,
                                            day: u16, month: u16, year: u16) -> anyhow::Result<ChannelInfo>{
        let exist = self.actors.iter().any(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase());
        if !exist{
            self.create_actor_channel(actor_id, state_psw).await?;
        }
        self.actors.iter_mut()
            .find(|ch| ch.actor_id().to_lowercase() == actor_id.to_lowercase()).unwrap()
            .create_daily_channel_in_date(state_psw, day, month, year).await

    }

    pub fn channel_info(&self) -> ChannelInfo{
        let info = self.channel.channel_address();
        ChannelInfo::new(info.0, info.1)
    }

    pub fn print_nested_channel_info(&self){
        let info = self.channel_info();
        let category = match self.category{
            Category::Trucks => "Trucks",
            Category::Scales => "Scales",
            Category::BioCells => "BioCells"
        };

        println!("  {} = {}:{}", category, info.channel_id, info.announce_id);
        self.actors.iter().for_each(|a| a.print_nested_channel_info());
        println!();
    }
}

impl CategoryChannel{
    async fn create_actor_channel(&mut self, actor_id: &str, state_psw: &str) -> anyhow::Result<()>{
        let found = self.actors.iter()
            .filter(|a| {actor_id.to_lowercase() == a.actor_id().to_string()})
            .count();
        if found > 0{
            return Err(anyhow::Error::msg("Actor channel with this id already exist"));
        }
        let mut actor_channel = ActorChannel::new(self.category.clone(), actor_id, self.mainnet);
        actor_channel.open(state_psw).await?;
        self.actors.push(actor_channel);
        Ok(())
    }
}
