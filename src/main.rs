#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::rng::Rng;
use embassy_nrf::{bind_interrupts, peripherals, rng, uarte};
use embassy_time::{Duration, Timer};
use heapless::String;
use rand::Rng as _;
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "embedded")]
use embedded_alloc::Heap;

#[cfg(feature = "embedded")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "embedded")]
#[global_allocator]
static HEAP: Heap = Heap::empty();

bind_interrupts!(struct Irqs {
    UARTE0_UART0 => uarte::InterruptHandler<peripherals::UARTE0>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

// Libwallet usage
use libwallet::{self, vault};
type Wallet = libwallet::Wallet<vault::Simple>;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let mut uart = uarte::Uarte::new(p.UARTE0, Irqs, p.P0_08, p.P0_06, config);

    info!("uarte initialized!");

    // Async API
    let mut bytes = [0; 4];
    rng.fill_bytes(&mut bytes).await;
    defmt::info!("Some random bytes: {:?}", bytes);

    // Message must be in SRAM
    // let mut buf: [u8; 80] = [0u8; 80];
    let mut palabra: String<512> = String::new();

    // Limpio pantalla y me pongo al inicio
    let _ = palabra.push(0x1B as char);
    unwrap!(uart.write(&palabra.clone().into_bytes()).await);
    let _ = palabra.push_str("[2J");
    unwrap!(uart.write(&palabra.clone().into_bytes()).await);
    let _ = palabra.push(0x1B as char);
    unwrap!(uart.write(&palabra.clone().into_bytes()).await);
    let _ = palabra.push_str("[H");
    unwrap!(uart.write(&palabra.clone().into_bytes()).await);
    palabra.clear();

    // Lanzo banner
    let _ = palabra.push_str("*********************************************\r\n");
    unwrap!(uart.write(&palabra.clone().into_bytes()).await);
    palabra.clear();
    let _ = palabra.push_str("   Usando bindgen y contando por la UART!\r\n");
    unwrap!(uart.write(&palabra.clone().into_bytes()).await);
    palabra.clear();
    let _ = palabra.push_str("*********************************************\r\n\n");
    unwrap!(uart.write(&palabra.clone().into_bytes()).await);
    palabra.clear();

    let phrase: &str = "";

    let vault = vault::Simple::generate(&mut rng);

    let mut wallet = Wallet::new(vault);
    wallet.unlock(None).await;
    let account = wallet.default_account();

    // Hago conteo
    loop {
        // info!("reading...");
        // unwrap!(uart.read(&mut buf).await);
        info!("writing...");
        let _ = palabra.push_str("Conteo: ");
        let random_num = rng.gen_range(0..9);
        info!("random_num: {}", random_num);
        let _ = palabra.push((49u8 + (random_num as u8)) as char);
        // let _ = palabra.push_str(&format!("\n\r address: {}", account));
        // info!("La cuenta pública: {}", account.public());
        defmt::info!("La cuenta pública: {:x}", account.public().as_ref());
        for byte in account.public().as_ref() {
            unwrap!(uart.write(&[*byte]).await);
        }

        // conteo = (conteo + 1) % 9;
        unwrap!(uart.write(&palabra.clone().into_bytes()).await);
        palabra.clear();
        Timer::after(Duration::from_millis(1000)).await;
    }
    // info!("wrote hello in uart!");
}

/* ********************************************************/
// #![no_std]
// #![no_main]
// #![feature(type_alias_impl_trait)]

// use defmt::*;
// use embassy_executor::Spawner;
// use embassy_nrf::gpio::{Level, Output, OutputDrive};
// use embassy_time::{Duration, Timer};
// use {defmt_rtt as _, panic_probe as _};

// // use wasm_runtime::{Runtime, Wasm};

// #[embassy_executor::main]
// async fn main(_spawner: Spawner) {
//     let p = embassy_nrf::init(Default::default());
//     let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

//     loop {
//         led.set_high();
//         Timer::after(Duration::from_millis(300)).await;
//         led.set_low();
//         Timer::after(Duration::from_millis(300)).await;
//     }
// }
/**********************************************************/

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
