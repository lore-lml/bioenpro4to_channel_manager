use serde::{Serialize, Deserialize};
use bioenpro4to_channel_manager::utils::current_time_secs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TruckMessage{
    license_plate: String,
    driver: String,
    timestamp: i64,
    max_weight: f32,
    last_update: TruckStepMessage
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TruckStepMessage{
    weight: f32,
    total_weight: f32,
    latitude: u32,
    longitude: u32,
    fuel: f32,
}

impl TruckStepMessage{
    pub fn new(weight: f32, total_weight: f32, latitude: u32, longitude: u32, fuel: f32) -> Self {
        TruckStepMessage { weight, total_weight, latitude, longitude, fuel }
    }
}

impl TruckMessage{
    pub fn new(license_plate: String, driver: String, timestamp: i64, max_weight: f32, last_update: TruckStepMessage) -> Self {
        TruckMessage { license_plate, driver, timestamp, max_weight, last_update }
    }
}


pub struct TruckMessageGenerator{
    license_plate: String,
    driver: String,
    timestamp: i64,
    max_weight: f32,
    cumulative_weight: f32,
    fuel: f32,
    latitude: u32,
    longitude: u32
}

impl TruckMessageGenerator{
    pub fn new(license_plate: String, driver: String) -> Self {
        TruckMessageGenerator {
            license_plate, driver,
            timestamp: current_time_secs(),
            max_weight: 500.0,
            cumulative_weight: 12.5,
            fuel: 100.0,
            latitude: 100,
            longitude: 100
        }
    }

    pub fn next(&mut self) -> TruckMessage{
        let step = TruckStepMessage::new(
            12.5, self.cumulative_weight,
            100, 100,
            self.fuel
        );
        let message = TruckMessage::new(
            self.license_plate.clone(), self.driver.clone(),
            self.timestamp, self.max_weight,
            step
        );

        self.timestamp += (1.5*3600.0) as i64;
        self.cumulative_weight += 12.5;
        self.fuel -= 12.5;
        self.latitude += 5;
        self.longitude += 5;

        message
    }
}
