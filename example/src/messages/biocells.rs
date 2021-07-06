use serde::{Serialize, Deserialize};
use crate::messages::ChannelReference;
use bioenpro4to_channel_manager::utils::current_time_secs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiocellMessage{
    plant: String,
    digestor_id: String,
    max_capacity: u32,
    start_timestamp: i64,
    last_measurement: MeasurementsMessage,
    scale_info: ScaleInfo
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeasurementsMessage{
    timestamp: i64,
    temperature: f32,
    humidity: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScaleInfo{
    weight: f32,
    msg_info: ChannelReference,
}

impl ScaleInfo{
    pub fn new(weight: f32, msg_info: ChannelReference) -> Self {
        ScaleInfo { weight, msg_info }
    }
}

impl MeasurementsMessage{
    pub fn new(timestamp: i64, temperature: f32, humidity: f32) -> Self {
        MeasurementsMessage { timestamp, temperature, humidity }
    }
}

impl BiocellMessage{
    pub fn new(plant: String, digestor_id: String, max_capacity: u32, start_timestamp: i64, last_measurement: MeasurementsMessage, scale_info: ScaleInfo) -> Self {
        BiocellMessage { plant, digestor_id, max_capacity, start_timestamp, last_measurement, scale_info }
    }
}


pub struct BiocellMessageGenerator{
    channel_id: String,
    announce_id: String,
    msg_id: String,
    plant: String,
    digestor_id: String,
    start_timestamp: i64,
    timestamp: i64,
    temperature: f32,
    humidity: f32,
}

impl BiocellMessageGenerator{
    pub fn new(channel_id: String, announce_id: String, msg_id: String, plant: String, digestor_id: String) -> Self {
        BiocellMessageGenerator {
            channel_id, announce_id, msg_id, plant, digestor_id,
            start_timestamp: current_time_secs(), timestamp: current_time_secs()+15*60,
            temperature: 18.4, humidity: 53.2 }
    }

    pub fn next(&mut self) -> BiocellMessage{
        let measure = MeasurementsMessage::new(
            self.timestamp, self.temperature, self.humidity
        );
        let scale_info = ScaleInfo::new(
            421.0,
            ChannelReference::new(
                self.channel_id.clone(), self.announce_id.clone(),
                self.msg_id.clone()
            )
        );
        let message = BiocellMessage::new(
            self.plant.clone(),
            self.digestor_id.clone(),
            304,
            self.start_timestamp,
            measure,
            scale_info
        );

        self.timestamp += 20*60;
        self.humidity += 1.1;
        self.temperature += 0.2;

        message
    }
}
