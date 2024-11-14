use esp_backtrace as _;
use esp_println as _;

use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::{RADIO_CLK, RNG, TIMG1, WIFI};
use esp_wifi::wifi::{
    new_with_mode, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
    WifiStaDevice, WifiState,
};

use core::mem::MaybeUninit;
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
pub fn init_heap() {
    // 32 * 1024 makes the Wifi Init Fail with EspNoMemErr (it looks like 32 isn't enough)
    const HEAP_SIZE: usize = 64 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            HEAP.as_mut_ptr() as *mut u8,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }
}

pub async fn initialize(
    timg1: TIMG1,
    rng: RNG,
    radio_clk: RADIO_CLK,
    wifi_p: WIFI,
) -> (
    &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    WifiController<'static>,
) {
    let timg = esp_hal::timer::timg::TimerGroup::new(timg1);
    let init = esp_wifi::init(
        esp_wifi::EspWifiInitFor::Wifi,
        timg.timer0,
        esp_hal::rng::Rng::new(rng),
        radio_clk,
    )
    .unwrap();
    let wifi = wifi_p;

    let (wifi_interface, controller) = new_with_mode(&init, wifi, WifiStaDevice).unwrap();
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

#[embassy_executor::task(pool_size = 2)]
pub async fn connection(mut controller: WifiController<'static>) {
    defmt::info!("start connection task");
    // defmt::info!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: "".try_into().unwrap(),
                password: "".try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            defmt::info!("Starting wifi");
            controller.start().await.unwrap();
            defmt::info!("Wifi started!");
        }
        defmt::info!("About to connect...");

        match controller.connect().await {
            Ok(_) => defmt::info!("Wifi connected!"),
            Err(e) => {
                defmt::info!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
pub async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
