use chrono::prelude::*;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use strategy::granularity::*;
use strategy::seq::*;
use strategy::strategy::simple::*;
use strategy::time::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let data = fs::read_to_string("./examples/data/EUR_USD_2019-01-01_2019-01-02.json")?;
    // println!("{}", data);
    let data: serde_json::Value = serde_json::from_str(data.as_str())?;

    // let v = get_value_from_data(
    //     &data,
    //     "S5",
    //     "2019-01-01 22:30:00",
    //     "EUR_USD",
    //     "bid",
    //     "close",
    // );
    // println!("{:?}", v);

    let start = "2019-01-01T22:00:00Z".parse::<DateTime<Utc>>().unwrap();
    // let st_start = "2019-01-01T23:00:00Z".parse::<DateTime<Utc>>().unwrap();
    let end = "2019-01-02T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    let start = Time::<S5>::new(start.timestamp());
    // let st_start = Time::<S5>::new(st_start.timestamp());
    let end = Time::<S5>::new(end.timestamp());

    let mut strategy = SimpleSmaCrossStrategy::new(SimpleStrategyBase {}, start, TransactionId(0));

    for time in start.range_to_end(end) {
        let dt: DateTime<Utc> = time.into();
        info!("{:?}", dt);

        update_source(&mut strategy, &data, dt);
        strategy.on_tick(dt);

        // if time >= st_start {
        //     strategy.on_tick(dt);
        // }
    }

    Ok(())
}

pub fn update_source(
    strategy: &mut SimpleSmaCrossStrategy,
    data: &serde_json::Value,
    dt: DateTime<Utc>,
) -> Option<()> {
    let dt_s = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    let mid_close = get_value_from_data(data, "S5", dt_s.as_str(), "EUR_USD", "mid", "close")?;
    let bid_close = get_value_from_data(data, "S5", dt_s.as_str(), "EUR_USD", "bid", "close")?;
    let ask_close = get_value_from_data(data, "S5", dt_s.as_str(), "EUR_USD", "ask", "close")?;
    strategy.update_source(dt, mid_close, bid_close, ask_close);
    Some(())
}

// #[derive(Deserialize)]
// struct Data {
//     S5: HashMap<String, >
// }

pub fn get_value_from_data(
    data: &serde_json::Value,
    granularity: &str,
    time: &str,
    instrument: &str,
    type_str: &str,
    name: &str,
) -> Option<f64> {
    data.as_object()?
        .get(granularity)?
        .as_object()?
        .get(time)?
        .as_object()?
        .get(instrument)?
        .as_object()?
        .get(type_str)?
        .as_object()?
        .get(name)?
        .as_f64()
}

// def get_value_from_data(data, granularity, time, instrument, type_str, name):
//     if granularity not in data:
//         return None
//     if time not in data[granularity]:
//         return None
//     if instrument not in data[granularity][time]:
//         return None
//     if type_str not in data[granularity][time][instrument]:
//         return None
//     if name not in data[granularity][time][instrument][type_str]:
//         return None
//     return data[granularity][time][instrument][type_str][name]
