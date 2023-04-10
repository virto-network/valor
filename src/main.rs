#![feature(type_alias_impl_trait)]
use clap::Parser;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use log::{info, warn};

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

#[embassy_executor::task(pool_size = 2)]
async fn plugin_run_dos(plugin: plugin::Plugin) {
    let mut count = 0;
    let rt = Runtime::with_defaults();
    let content_plugin = plugin.get_plugin();
    let app = rt.load(content_plugin).unwrap();
    rt.run(&app).unwrap();
}

#[embassy_executor::task(pool_size = 2)]
async fn plugin_run(plugin: plugin::Plugin) {
    // async fn plugin_run(my_num: u32) {
    // for _ in 0..5 {
    //     println!("Hey from concurrent pool {}", my_num);
    //     Timer::after(Duration::from_millis(1000)).await;
    // }

    let mut count = 0;
    let rt = Runtime::with_defaults();
    let content_plugin = plugin.get_plugin();
    let app = rt.load(content_plugin).unwrap();
    rt.run(&app).unwrap();
}

#[embassy_executor::task]
async fn run(paths: Vec<String>, all_active: bool) {
    let mut vec_plugins: Vec<plugin::Plugin> = plugin::Plugin::new_vec(paths.clone(), all_active); // Check how to do it well done
    let mut vec_active_plugins: Vec<usize> = Vec::new();

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

    // let mut sp_vec = vec![];
    let sp_vec = [
        Spawner::for_current_executor().await,
        Spawner::for_current_executor().await,
    ];

    let _ = sp_vec[0].spawn(plugin_run(vec_plugins[0].clone()));
    println!("************************************************** Estoy acá!");
    let _ = sp_vec[1].spawn(plugin_run_dos(vec_plugins[1].clone()));

    // let mut counter = 0;
    // for plugin in vec_plugins {
    //     if plugin.active {
    //         sp_vec[counter] = Spawner::for_current_executor().await;

    //         // let sp = Spawner::for_current_executor().await;
    //         info!("Ejecutando plugin: {}", plugin.name);
    //         sp_vec[counter].spawn(plugin_run(plugin));
    //         // handles.push(handle);
    //         counter = counter + 1;
    //     }
    // }

    // let sp = Spawner::for_current_executor().await;

    // let sp = [
    //     Spawner::for_current_executor().await,
    //     Spawner::for_current_executor().await,
    // ];
    // for num in 0..5 {
    //     sp[0].spawn(plugin_run(num));
    // }

    // for num2 in 5..10 {
    //     sp[1].spawn(plugin_run(num2));
    // }

    info!("Después de threads");

    // for t in handles {
    //     info!("Join de threads!");
    //     t.join().unwrap();
    // }

    warn!("Finished embassy run!");
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Parse cli inputs (go to parsero)
    let args = parsero::Args::parse();
    if !args.quiet {
        print_banner();
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
