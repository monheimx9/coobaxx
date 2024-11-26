use defmt::Format;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
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

#[derive(PartialEq, Clone, Copy, Debug, Format)]
pub enum CurrentScreen {
    Starting,
    Messaging,
    Alert,
}
impl Default for CurrentScreen {
    fn default() -> Self {
        CurrentScreen::Starting
    }
}

type Ssd1306IDisplay<'a> = Ssd1306<
    I2CInterface<&'a mut I2c<'static, Async>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub async fn draw_display(i2c: &mut I2c<'static, Async, AnyI2c>, screen: &CurrentScreen) {
    defmt::info!("Display drawing");
    let interface = I2CDisplayInterface::new(i2c);
    let mut display: Ssd1306IDisplay =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
    display.init().unwrap();
    let bouding_box = display.bounding_box();
    display.clear(BinaryColor::Off).unwrap();
    let style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_off = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let _ = Text::with_baseline("Anouar!", Point::new(2, 2), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    display.flush().unwrap();
}
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
