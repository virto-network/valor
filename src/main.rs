#![feature(type_alias_impl_trait)]
// Johan: pregunta a Dani: esto no debería ir en el módulo parsero/mod.rs y no acá?
use clap::Parser;

use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};
// use log::*;

use std::io::stdin;
use wasm_runtime::{Runtime, Wasm};

mod parsero;
mod plugin;

#[embassy_executor::task]
async fn run(args: Vec<String>) {
    let map_plugins = plugin::Plugin::new_map(&args);

    let rt = Runtime::with_defaults();

    println!("Please select which plugins do you want to load:");
    for (index, value) in args.iter().enumerate() {
        println!("{} {}", index, value);
    }

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    let content_plugin = map_plugins.get(input).unwrap().get_plugin();

    let app = rt.load(content_plugin).unwrap();
    rt.run(&app).unwrap();
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Parse cli inputs (go to parsero)
    let args = parsero::Args::parse();

    if false == args.check_plugin_paths() {
        panic!("Please check the provided paths");
    }

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    spawner.spawn(run(args.plugin_path)).unwrap();
}
