use alloc::format;
use defmt::Format;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Alignment, Baseline, Text};
use esp_hal::Async;
use heapless::String;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, I2CDisplayInterface};

use esp_hal::i2c::master::AnyI2c;
use esp_hal::i2c::master::I2c;

use esp_backtrace as _;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306;

use crate::task::state::SCREEN_STATE_MUTEX;
mod messaging;

#[derive(Debug, Default, PartialEq, Copy, Clone, Format)]
pub enum CurrentScreen {
    Starting,
    #[default]
    Messaging,
    MessageSent,
    Alert,
    Error,
}

type Ssd1306IDisplay<'a, T> =
    Ssd1306<I2CInterface<&'a mut I2c<'static, Async>>, DisplaySize128x64, BufferedGraphicsMode<T>>;

pub async fn draw_display(i2c: &mut I2c<'static, Async, AnyI2c>) {
    let screen_data_guard = SCREEN_STATE_MUTEX.lock().await;
    let screen_data = screen_data_guard.as_ref().unwrap();

    let interface = I2CDisplayInterface::new(i2c);
    let mut display: Ssd1306IDisplay<DisplaySize128x64> =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
    display.init().unwrap();
    match screen_data.current_screen {
        CurrentScreen::Messaging => messaging::draw(&mut display, screen_data),
        _ => {}
    }
    display.flush().unwrap();
    drop(screen_data_guard);
}
