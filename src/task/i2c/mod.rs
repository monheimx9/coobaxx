use defmt::Format;
use esp_backtrace as _;

pub mod display;

use display::{draw_display, CurrentScreen};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use esp_hal::{
    i2c::master::{AnyI2c, I2c},
    Async,
};

use super::task_messages::I2C_MANAGER_SIGNAL;

pub type I2cMaster = I2c<'static, Async, AnyI2c>;
type I2cManager = Mutex<CriticalSectionRawMutex, Option<I2cMaster>>;
pub static I2C_MANAGER: I2cManager = Mutex::new(None);

#[derive(PartialEq, Debug, Format)]
pub enum I2CDevice {
    Ssd1306Display,
    RtcDs3231(RtcAction),
}

#[derive(PartialEq, Debug, Format)]
pub enum RtcAction {
    Read,
    Write,
}

#[embassy_executor::task(pool_size = 1)]
pub async fn i2c_manager(i2c_m: I2cMaster) {
    defmt::info!("I2C Manager starting");
    {
        *(I2C_MANAGER.lock().await) = Some(i2c_m);
    }
    loop {
        '_i2c_manager: {
            defmt::info!("Waiting for I2C Signal");
            if I2C_MANAGER_SIGNAL.signaled() {
                I2C_MANAGER_SIGNAL.reset();
                defmt::info!("Signal reseted");
            }
            let i2c_dev = I2C_MANAGER_SIGNAL.wait().await;
            let mut i2c = I2C_MANAGER.lock().await;
            let i2c_guard = i2c.as_mut().unwrap();

            match i2c_dev {
                I2CDevice::Ssd1306Display => {
                    defmt::info!("Drawing display");
                    draw_display(i2c_guard).await;
                    defmt::info!("Display drawed");
                }

                I2CDevice::RtcDs3231(action) => {
                    // draw_display(i2c_guard).await;
                }
            }
            // drop(i2c_guard);
        }
    }
}
