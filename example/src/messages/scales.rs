use serde::{Serialize, Deserialize};
use crate::messages::ChannelReference;
use bioenpro4to_channel_manager::utils::current_time_secs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScaleMessage{
    plant: String,
    timestamp: i64,
    truck_info: TruckInfo
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TruckInfo{
    weight: f32,
    driver: String,
    license_plate: String,
    msg_info: ChannelReference
}

impl TruckInfo{
    pub fn new(weight: f32, driver: String, license_plate: String, msg_info: ChannelReference) -> Self {
        TruckInfo { weight, driver, license_plate, msg_info }
    }
}

impl ScaleMessage{
    pub fn new(plant: String, timestamp: i64, truck_info: TruckInfo) -> Self {
        ScaleMessage { plant, timestamp, truck_info }
    }
}


pub struct ScaleMessageGenerator{
    channel_id: String,
    announce_id: String,
    msg_id: String,
    plant: String,
    timestamp: i64,
    license_plate: String,
    driver: String,
}

impl ScaleMessageGenerator{
    pub fn new(channel_id: String, announce_id: String, msg_id: String, plant: String, license_plate: String, driver: String) -> Self {
        ScaleMessageGenerator {
            channel_id, announce_id,  msg_id,
            plant,
            timestamp: current_time_secs(),
            license_plate, driver
        }
    }

    pub fn next(&mut self) -> ScaleMessage{
        let truck_info = TruckInfo::new(
            320.0,
            self.driver.clone(),
            self.license_plate.clone(),
            ChannelReference::new(
                self.channel_id.clone(), self.announce_id.clone(),
                self.msg_id.clone()
            )
        );
        let message = ScaleMessage::new(
            self.plant.clone(),
            self.timestamp,
            truck_info
        );

        self.timestamp += 3600;
        message
    }
}
