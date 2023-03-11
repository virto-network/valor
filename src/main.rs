#![feature(type_alias_impl_trait)]

use std::{collections::HashMap, env, fs, io::stdin};

use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};
// use log::*;
use wasm_runtime::{Runtime, Wasm};

#[derive(Debug)]
struct Plugin<'a> {
    name: &'a str,
    content: Vec<u8>,
}

impl<'a> Plugin<'a> {
    fn new(name: &'a str, content: Vec<u8>) -> Self {
        Plugin { name, content }
    }
    fn get_plugin(&self) -> &[u8] {
        &self.content
    }
}

#[embassy_executor::task]
async fn run(args: Vec<String>) {
    // let mut app = Vec::new();
    // stdin().read_to_end(&mut app).expect("WASM read");
    let mut vec_plugins = HashMap::<&str, Plugin>::new();

    for arg in args.iter() {
        let content_plugin = fs::read(&arg).expect("Epic Fail!, The file doesn't exist!. :(");
        let plugin = Plugin::new(arg.as_str(), content_plugin);
        vec_plugins.insert(arg.as_str(), plugin);
    }

    let rt = Runtime::with_defaults();

    println!("Please select which plugins do you want to load:");
    for (index, value) in args.iter().enumerate() {
        println!("{} {}", index, value);
    }

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    let content_plugin = vec_plugins.get(input).unwrap().get_plugin();

    let app = rt.load(content_plugin).unwrap();
    rt.run(&app).unwrap();
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let args: Vec<String> = env::args().skip(1).collect();
    // let mut plugins_paths = Vec::<&str>::new();

    // for i in args {
    //     plugins_paths.push(&i);
    // }

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    spawner.spawn(run(args)).unwrap();
}
