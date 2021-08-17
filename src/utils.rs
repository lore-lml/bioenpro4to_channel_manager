pub extern crate serde;
pub use iota_streams_lib::utility::iota_utility::{create_encryption_nonce, create_encryption_key, hash_string};
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

pub fn check_date_format(date: &str) -> bool{
    let re = regex::Regex::new(
        r"^([0-2][0-9]|(3)[0-1])(/)(((0)[0-9])|((1)[0-2]))(/)\d{4}$"
    ).unwrap();
    re.is_match(date)
}
