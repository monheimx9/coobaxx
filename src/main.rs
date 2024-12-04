#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::I2c;
use esp_hal::{gpio::*, rng::Rng};
use esp_wifi::EspWifiController;
extern crate alloc;

use {defmt_rtt as _, esp_backtrace as _};

use embassy_executor::Spawner;
use embassy_time::Timer;

mod constant;
pub use constant::*;

mod task;
use task::{button::*, i2c::*, mqtt::*, orchestrate::*};

pub mod utils;
use utils::mk_static;

mod init_board;
use init_board::{connection, initialize_wifi_stack, net_task};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    esp_alloc::heap_allocator!(64 * 1024);
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg0.timer0);

    spawner.must_spawn(orchestrator());
    let gpio32: GpioPin<19> = peripherals.GPIO19;
    let gpio33: GpioPin<20> = peripherals.GPIO20;

    let sda = gpio32;
    let scl = gpio33;
    let i2c: I2cMaster = I2c::new(peripherals.I2C0, Default::default())
        .with_sda(sda)
        .with_scl(scl)
        .into_async();

    spawner.spawn(i2c_manager(i2c)).ok();

    let wifi_init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(
            timg1.timer0,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
        )
        .unwrap()
    );

    let (stack, controller) = initialize_wifi_stack(wifi_init, peripherals.WIFI).await;

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(stack)).ok();

    loop {
        defmt::info!("Waiting to get IP address...");
        if let Some(config) = stack.config_v4() {
            defmt::info!("Got IP: {}", config.address);
            break;
        }
        Timer::after_secs(5).await;
    }

    let select_btn = Input::new(peripherals.GPIO10, Pull::Down);
    let send_btn = Input::new(peripherals.GPIO11, Pull::Down);

    spawner.spawn(scheduler()).ok();
    spawner.spawn(handler_select_btn(select_btn)).ok();
    spawner.spawn(handler_send_btn(send_btn)).ok();
    spawner.spawn(mqtt_manager(stack)).ok();

    loop {
        Timer::after_secs(5).await;
    }
}
