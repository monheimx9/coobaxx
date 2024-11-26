#![no_std]
#![no_main]

mod init_board;

use esp_hal::prelude::*;
use esp_hal::{gpio::*, rng::Rng};

use esp_hal::i2c::master::I2c;

use esp_wifi::EspWifiController;
use init_board::{connection, init_heap, initialize_wifi_stack, net_task};

use esp_backtrace as _;
use esp_println as _;

pub mod utils;

use embassy_executor::Spawner;
use embassy_time::Timer;

use esp_backtrace as _;
use task::{
    button::handler_clear_btn,
    i2c::{i2c_manager, I2cMaster},
    mqtt::mqtt_manager,
    orchestrate::{orchestrator, scheduler},
};
use utils::mk_static;

extern crate alloc;

mod task;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();
    let peripherals = esp_hal::init(Default::default());
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    spawner.must_spawn(orchestrator());
    let gpio32: GpioPin<32> = peripherals.GPIO32;
    let gpio33: GpioPin<33> = peripherals.GPIO33;

    let sda = gpio32;
    let scl = gpio33;
    let i2c: I2cMaster = I2c::new(peripherals.I2C1, Default::default())
        .with_sda(sda)
        .with_scl(scl)
        .into_async();

    spawner.spawn(i2c_manager(i2c)).ok();

    let wifi_init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(
            timg0.timer1,
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

    let btn = Input::new(peripherals.GPIO14, Pull::Down);

    spawner.spawn(scheduler()).ok();
    spawner.spawn(handler_clear_btn(btn)).ok();
    spawner.spawn(mqtt_manager(stack)).ok();

    loop {
        Timer::after_secs(5).await;
    }
}
