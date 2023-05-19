#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, peripherals, uarte};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UARTE0_UART0 => uarte::InterruptHandler<peripherals::UARTE0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let mut uart = uarte::Uarte::new(p.UARTE0, Irqs, p.P0_08, p.P0_06, config);

    info!("uarte initialized!");

    // Message must be in SRAM
    let mut buf = [0; 8];
    buf.copy_from_slice(b"Hello!\r\n");

    unwrap!(uart.write(&buf).await);
    info!("wrote hello in uart!");

    loop {
        info!("reading...");
        unwrap!(uart.read(&mut buf).await);
        info!("writing...");
        unwrap!(uart.write(&buf).await);
    }
}

// #![no_std]
// #![no_main]
// #![feature(type_alias_impl_trait)]

// use clap::Parser;
// use defmt::*;
// use embassy_executor::Spawner;
// use embassy_nrf::{bind_interrupts, peripherals, uarte};
// use log::{info, warn};
// use {defmt_rtt as _, panic_probe as _};
// // use embassy_time::{Duration, Timer};

// use std::io::stdin;
// use std::thread;
// use wasm_runtime::{Runtime, Wasm};

// mod constants;
// mod parsero;
// mod plugin;
// mod utils;

// bind_interrupts!(struct Irqs {
//     UARTE0_UART0 => uarte::InterruptHandler<peripherals::UARTE0>;
// });

// #[embassy_executor::task(pool_size = 5)]
// async fn run(paths: Vec<String>, all_active: bool) {
//     // Initialize NRF52
//     let p = embassy_nrf::init(Default::default());
//     let mut config = uarte::Config::default();
//     config.parity = uarte::Parity::EXCLUDED;
//     config.baudrate = uarte::Baudrate::BAUD115200;

//     let mut uart = uarte::Uarte::new(p.UARTE0, Irqs, p.P0_08, p.P0_06, config);

//     info!("uarte initialized!");
//     // </> End initialization NRF52

//     // Optional usage of map_plugins
//     // let map_plugins = plugin::Plugin::new_map(&paths, all_active);
//     let mut vec_plugins: Vec<plugin::Plugin> = plugin::Plugin::new_vec(paths.clone(), all_active);
//     let mut _vec_active_plugins: Vec<usize> = Vec::new();
//     let mut handles = vec![];

//     if !all_active {
//         println!("Please select the index of which plugins do you want to load using comma to separate id's like this:");
//         println!("\t id1,id2,id3");
//         for (index, value) in paths.iter().enumerate() {
//             println!("[{}] -> {}", index, value);
//         }
//         let mut input = String::new();
//         stdin().read_line(&mut input).unwrap();
//         _vec_active_plugins = input
//             .trim()
//             .split(',')
//             .map(|s| s.parse().unwrap())
//             .collect();

//         _vec_active_plugins.sort();
//         _vec_active_plugins.dedup();

//         // Activate plugins
//         for key in _vec_active_plugins {
//             if key < vec_plugins.len() {
//                 vec_plugins[key].active = true;
//             } else {
//                 warn!("Wrong value provided: {}!. Skipped plugin.", key);
//             }
//         }
//     }

//     for plugin in vec_plugins {
//         if plugin.active {
//             let handle = thread::spawn(move || {
//                 info!("Loading wasi app from {}", plugin.name);
//                 let rt = Runtime::with_defaults();
//                 let content_plugin = plugin.get_plugin();
//                 let app = rt.load(content_plugin).unwrap();
//                 rt.run(&app).unwrap();
//             });
//             handles.push(handle);
//         }
//     }

//     info!("Después de threads");

//     // ToDo: Detect sigkills in Unix, and determinate on embedded
//     for t in handles {
//         info!("Join de threads!");
//         t.join().unwrap();
//     }

//     warn!("Finished embassy run!");
// }

// #[embassy_executor::main]
// async fn main(spawner: Spawner) {
//     // Parse cli inputs
//     let args = parsero::Args::parse();
//     if !args.quiet {
//         utils::print_banner(constants::BANNER);
//     }

//     if false == args.check_plugin_paths() {
//         panic!("Please check the provided paths");
//     }

//     env_logger::builder()
//         .filter_level(log::LevelFilter::Info)
//         .format_timestamp_nanos()
//         .init();

//     spawner
//         .spawn(run(args.plugin_path, args.all_active))
//         .unwrap();

//     println!("Finished program!");
// }
