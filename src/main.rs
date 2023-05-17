#![feature(type_alias_impl_trait)]
use clap::Parser;

use embassy_executor::Spawner;
use log::{info, warn};
// use embassy_time::{Duration, Timer};

use std::io::stdin;
use std::process::Command;
use std::thread;
use wasm_runtime::{Runtime, Wasm};

mod constants;
mod parsero;
mod plugin;
mod utils;

#[embassy_executor::task(pool_size = 5)]
async fn run(paths: Vec<String>, all_active: bool) {
    // let map_plugins = plugin::Plugin::new_map(&paths, all_active);
    let mut vec_plugins: Vec<plugin::Plugin> = plugin::Plugin::new_vec(paths.clone(), all_active); // Check how to do it well done
    let mut vec_active_plugins: Vec<usize> = Vec::new();
    let mut handles = vec![];

    if !all_active {
        println!("Please select the index of which plugins do you want to load using comma to separate id's like this:");
        println!("\t id1,id2,id3");
        for (index, value) in paths.iter().enumerate() {
            println!("[{}] -> {}", index, value);
        }
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        vec_active_plugins = input
            .trim()
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        vec_active_plugins.sort();
        vec_active_plugins.dedup();

        // Activate plugins
        for key in vec_active_plugins {
            if key < vec_plugins.len() {
                vec_plugins[key].active = true;
            } else {
                // Replace with log message
                warn!("Wrong value provided: {}!. Skipped plugin.", key);
            }
        }
    }

    for plugin in vec_plugins {
        if plugin.active {
            // vec_rt.push(Runtime::with_defaults());
            let handle = thread::spawn(move || {
                info!("Loading wasi app from {}", plugin.name);
                let rt = Runtime::with_defaults();
                let content_plugin = plugin.get_plugin();
                let app = rt.load(content_plugin).unwrap();
                rt.run(&app).unwrap();
            });
            handles.push(handle);
        }
    }

    info!("Después de threads");

    // Detectar kill signal
    // Si detectada kill signal entonces thread.join aquí.
    // for t in handles {
    //     t.join();
    // }
    for t in handles {
        info!("Join de threads!");
        t.join().unwrap();
    }

    warn!("Finished embassy run!");
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Parse cli inputs (go to parsero)
    let args = parsero::Args::parse();
    if !args.quiet {
        utils::print_banner(constants::BANNER);
    }

    if false == args.check_plugin_paths() {
        panic!("Please check the provided paths");
    }

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    spawner
        .spawn(run(args.plugin_path, args.all_active))
        .unwrap();
}