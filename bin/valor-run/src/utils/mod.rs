// Utils directory
use std::process::Command;

pub fn print_banner(banner: &str) {
    // Clean screen
    let output = Command::new("clear")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    print!("{}", String::from_utf8_lossy(&output.stdout));

    // Print banner
    println!("{}", banner);
}
