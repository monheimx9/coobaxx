#![no_std]
#![no_main]

mod init_board;
use core::cell::RefCell;

use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use init_board::{connection, init_heap, initialize, net_task};

use esp_backtrace as _;
use esp_println as _;

mod mqtt;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use mqtt::send_mqtt_message;
use task::{
    display::{do_something_else, perform, CurrentScreen},
    state::AppState,
};

extern crate alloc;

mod task;
// use task::display::show_something;
static APPSTATE: Mutex<CriticalSectionRawMutex, RefCell<AppState>> =
    Mutex::new(RefCell::new(AppState {
        screen: Some(CurrentScreen::Home),
    }));

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    init_heap();
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let (stack, controller) = initialize(
        peripherals.TIMG1,
        peripherals.RNG,
        peripherals.RADIO_CLK,
        peripherals.WIFI,
    )
    .await;
    spawner.spawn(perform(&APPSTATE)).ok();
    spawner.spawn(do_something_else(&APPSTATE)).ok();

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(stack)).ok();

    loop {
        defmt::info!("Waiting to get IP address...");
        if let Some(config) = stack.config_v4() {
            defmt::info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    send_mqtt_message(stack).await;

    loop {
        Timer::after(Duration::from_secs(10)).await;
    }
}
