use core::cell::RefCell;

use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use embassy_time::Timer;

use crate::task::state::AppState;
use esp_backtrace as _;
use esp_println as _;

#[derive(Clone, Copy, Debug)]
pub enum CurrentScreen {
    Home,
    Time,
    Alert,
}
impl Default for CurrentScreen {
    fn default() -> Self {
        CurrentScreen::Home
    }
}
// &embassy_sync::mutex::Mutex<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, core::cell::RefCell<task::state::AppState>>`
#[embassy_executor::task]
pub async fn perform(rcapp: &'static Mutex<CriticalSectionRawMutex, RefCell<AppState>>) -> ! {
    loop {
        critical_section::with(|cs| rcapp.borrow(cs).take().change_screen());
        defmt::info!("Screen changed !");

        Timer::after_secs(30).await;
    }
}

#[embassy_executor::task]
pub async fn do_something_else(
    rcapp: &'static Mutex<CriticalSectionRawMutex, RefCell<AppState>>,
) -> ! {
    loop {
        critical_section::with(|cs| rcapp.borrow(cs).take().change_screen());

        defmt::info!("Screen changed ! From do_something_else !!!!");

        Timer::after_secs(30).await;
    }
}

// use embedded_graphics::mono_font::ascii::FONT_6X10;
// use embedded_graphics::mono_font::MonoTextStyleBuilder;
// use embedded_graphics::pixelcolor::BinaryColor;
// use embedded_graphics::prelude::Point;
// use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
// use embedded_graphics::text::{Baseline, Text};
// use embedded_hal_bus::i2c::RefCellDevice;
// use esp_hal::i2c::*;
// use esp_hal::peripherals::Peripherals;
// use ssd1306::{prelude::*, Ssd1306Async};
//
// use ssd1306::size::DisplaySize128x64;
// use ssd1306::I2CDisplayInterface;
//
// pub async fn show_something() {
//     let periph = Peripherals::steal();
//     let sda = periph.pins.gpio32;
//     let scl = periph.pins.gpio33;
//     let config = I2cConfig::new().baudrate(100.kHz().into());
//     let i2c = RefCell::new(I2cDriver::new(periph.i2c0, sda, scl, &config)?);
//
//     let interface = I2CDisplayInterface::new(RefCellDevice::new(&i2c));
//     let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
//         .into_buffered_graphics_mode();
//     display.init().unwrap();
//
//     let style = PrimitiveStyleBuilder::new()
//         .stroke_width(1)
//         .stroke_color(BinaryColor::On)
//         .build();
//
//     let text_style = MonoTextStyleBuilder::new()
//         .font(&FONT_6X10)
//         .text_color(BinaryColor::On)
//         .build();
//     let text_off = MonoTextStyleBuilder::new()
//         .font(&FONT_6X10)
//         .text_color(BinaryColor::Off)
//         .build();
//
//     Text::with_baseline("Anouar!", Point::new(2, 2), text_style, Baseline::Top)
//         .draw(&mut display)
//         .unwrap();
//
//     Text::with_baseline(
//         "On t'entend pas putain!",
//         Point::new(2, 16),
//         text_style,
//         Baseline::Top,
//     )
//     .draw(&mut display)
//     .unwrap();
//     Rectangle::new(Point::new(0, 0), Size::new(127, 63))
//         .into_styled(style)
//         .draw(&mut display)
//         .unwrap();
//
//     display.flush().unwrap();
// }
