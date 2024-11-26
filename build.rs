use std::{env, fs::File, io::Write, path::Path};

extern crate dotenvy;
fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");

    // println!("cargo:rustc-link-arg-bins=-Trom_functions.x");
    wifi_secret();
}

fn wifi_secret() {
    dotenvy::dotenv().unwrap();
    let ssid = env::var("SSID").unwrap();
    let passw = env::var("PASSWORD").unwrap();
    let dest_path = Path::new("src/").join("wifi_secret.rs");
    let mut f = File::create(dest_path).unwrap();

    writeln!(f, "pub const SSID: & str = \"{}\";", ssid).unwrap();
    writeln!(f, "pub const PASSWORD: & str = \"{}\";", passw).unwrap();
}
