use esp_backtrace as _;
use esp_println as _;

use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
use esp_wifi::{
    wifi::{
        new_with_mode, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
        WifiStaDevice, WifiState,
    },
    EspWifiController,
};

use crate::utils::mk_static;

use core::mem::MaybeUninit;

pub fn init_heap() {
    // 32 * 1024 makes the Wifi Init Fail with EspNoMemErr (it looks like 32 isn't enough)
    const HEAP_SIZE: usize = 72 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            HEAP.as_mut_ptr() as *mut u8,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }
}
pub type WifiStack = &'static Stack<WifiDevice<'static, WifiStaDevice>>;

// pub const SSID: & str = "secret_ssid";
// pub const PASSWORD: & str = "secret_password";
// Those are included from that file, look as build.rs
include!("wifi_secret.rs");

pub async fn initialize_wifi_stack(
    esp_wc: &'static EspWifiController<'static>,
    wifi: WIFI,
) -> (WifiStack, WifiController<'static>) {
    let (wifi_interface, controller) = new_with_mode(esp_wc, wifi, WifiStaDevice).unwrap();
    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = 1234;
    let stack = &*mk_static!(
        Stack<WifiDevice<'_, WifiStaDevice>>,
        Stack::new(
            wifi_interface,
            config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            seed,
        )
    );
    (stack, controller)
}

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
pub async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
