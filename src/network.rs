use esp_backtrace as _;

use embassy_net::Runner;
use embassy_time::{Duration, Timer};
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
    WifiState,
};

// pub const SSID: & str = "secret_ssid";
// pub const PASSWORD: & str = "secret_password";
// Those are included from that file, look as build.rs
include!("wifi_secret.rs");

#[embassy_executor::task(pool_size = 1)]
pub async fn connection(mut controller: WifiController<'static>) {
    defmt::info!("start connection task");
    // defmt::info!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            defmt::info!("Starting wifi");
            controller.start_async().await.unwrap();
            defmt::info!("Wifi started!");
        }
        defmt::info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => defmt::info!("Wifi connected!"),
            Err(e) => {
                defmt::info!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}
