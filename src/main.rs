#![feature(type_alias_impl_trait)]

use std::io::{stdin, Read};

use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};
// use log::*;
use wasm_runtime::{Runtime, Wasm};

#[embassy_executor::task]
async fn run() {
    let mut app = Vec::new();
    stdin().read_to_end(&mut app).expect("WASM read");

    let rt = Runtime::with_defaults();
    let app = rt.load(&app).unwrap();
    rt.run(&app).unwrap();
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    spawner.spawn(run()).unwrap();
}
