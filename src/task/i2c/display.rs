use defmt::Format;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Baseline, Text};
use esp_hal::Async;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, I2CDisplayInterface};

use esp_hal::i2c::master::AnyI2c;
use esp_hal::i2c::master::I2c;

use esp_backtrace as _;
use esp_println as _;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306;

use crate::task::state::{ScreenData, SCREEN_STATE_MUTEX, STATE_MANAGER_MUTEX};

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
    display.set_brightness(Brightness::DIM).unwrap();
    let bounding_box = display.bounding_box();
    display.clear(BinaryColor::Off).unwrap();
    let style_on = PrimitiveStyle::with_fill(BinaryColor::On);
    let style_off = PrimitiveStyle::with_fill(BinaryColor::Off);
    let text_on = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_off = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::Off)
        .build();

    let items = &screen_data.items;
    let selected = &screen_data.selected_item;
    defmt::info!("Selected item {}", selected);
    let x = 2;
    let mut y = 1;
    for i in 0..screen_data.items.len() {
        let style = if i as u8 == *selected {
            Rectangle::new(Point::new(x, y - 1), Size::new(80, 12))
                .into_styled(style_on)
                .draw(&mut display)
                .unwrap();
            text_off
        } else {
            Rectangle::new(Point::new(x, y - 1), Size::new(80, 12))
                .into_styled(style_off)
                .draw(&mut display)
                .unwrap();
            text_on
        };
        Text::with_baseline(items[i].as_str(), Point::new(x, y), style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        y += 12;
    }
    display.flush().unwrap();
    drop(screen_data_guard);
}

async fn messaging_screen() {}
