pub extern crate serde;
use chrono::{Local, NaiveDateTime, Datelike};

pub fn current_time_millis() -> i64{
    Local::now().timestamp_millis()
}

pub fn current_time_secs() -> i64{
    Local::now().timestamp()
}

pub fn timestamp_to_date(timestamp: i64, millis: bool) -> NaiveDateTime{
    let (sec, nsec): (i64, u32) = if millis{
        (timestamp / 1000, (timestamp % 1000) as u32 * 1000000)
    }else{
        (timestamp, 0)
    };
    NaiveDateTime::from_timestamp(sec, nsec)
}

pub fn timestamp_to_date_string(timestamp: i64, millis: bool) -> String{
    let date = timestamp_to_date(timestamp, millis);
    format!("{:02}/{:02}/{}", date.day(), date.month(), date.year())
}

pub fn create_encryption_key(string_key: &str) -> [u8; 32]{
    iota_streams_lib::utility::iota_utility::create_encryption_key(string_key)
}

pub fn create_encryption_nonce(string_nonce: &str) -> [u8; 24]{
    iota_streams_lib::utility::iota_utility::create_encryption_nonce(string_nonce)
}
