#![feature(type_alias_impl_trait)]
// Johan: pregunta a Dani: esto no debería ir en el módulo parsero/mod.rs y no acá?
use clap::Parser;

use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};
// use log::*;

use std::io::stdin;
use std::process::Command;
use wasm_runtime::{Runtime, Wasm};

mod parsero;
mod plugin;

fn print_banner() {
    // Clean screen
    let output = Command::new("clear")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    print!("{}", String::from_utf8_lossy(&output.stdout));

    // Set banner
    let banner = r#"
*********************************************************
 __    __)      _____    _____       ______)      ___  
(, )  /        (, /     (, /   )    (, /        /(,  ) 
   | /           /        /__ /       /        /    /  
   |/        ___/__    ) /   \_    ) /        /    /   
   |       (__ /      (_/         (_/        (___ /    
                                                       
*********************************************************
"#;

    // Print banner
    println!("{banner}");
}

#[embassy_executor::task]
async fn run(paths: Vec<String>, all_active: bool) {
    // let map_plugins = plugin::Plugin::new_map(&paths, all_active);
    let mut vec_plugins: Vec<plugin::Plugin> = plugin::Plugin::new_vec(&paths, all_active);
    let mut vec_active_plugins: Vec<usize> = Vec::new();

    // let rt = Runtime::with_defaults();
    let mut vec_rt: Vec<Runtime> = Vec::new();

    if !all_active {
        println!("Please select the id's of which plugins do you want to load using comma to separate id's like this:");
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
                println!("Wrong value provided: {}!. Skipped plugin.", key);
            }
        }
    }

    for plugin in vec_plugins {
        if plugin.active {
            vec_rt.push(Runtime::with_defaults());
            println!("Loading wasi app from {}", plugin.name);
            let content_plugin = plugin.get_plugin();
            let app = vec_rt[vec_rt.len() - 1].load(content_plugin).unwrap();
            vec_rt[vec_rt.len() - 1].run(&app).unwrap();
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Parse cli inputs (go to parsero)
    print_banner();
    let args = parsero::Args::parse();

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
