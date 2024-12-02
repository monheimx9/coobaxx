use std::{env, fs::File, io::Write, path::Path};

extern crate dotenvy;
fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    // println!("cargo:rustc-link-arg-bins=-Trom_functions.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // println!("cargo:rustc-link-arg-bins=-Trom_functions.x");
    dotenvy::dotenv().unwrap();
    wifi_secret();
    broadcast_options();
}

fn wifi_secret() {
    let ssid = env::var("SSID").unwrap();
    let passw = env::var("PASSWORD").unwrap();
    let dest_path = Path::new("src/").join("wifi_secret.rs");
    let mut f = File::create(dest_path).unwrap();

    writeln!(f, "pub const SSID: & str = \"{}\";", ssid).unwrap();
    writeln!(f, "pub const PASSWORD: & str = \"{}\";", passw).unwrap();
}

fn broadcast_options() {
    let current = env::var("DEVICE_NAME").unwrap();
    let dest_path = Path::new("src/").join("broadcast_options.rs");
    let mut f = File::create(dest_path).unwrap();
    writeln!(f, "pub const CURRENT_DEVICE_NAME: &str = \"{}\";", current).unwrap();
}
