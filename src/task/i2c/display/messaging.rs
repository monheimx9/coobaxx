use ssd1306::size::DisplaySize128x64;

use crate::task::state::ScreenData;

use super::*;

pub fn draw(display: &mut Ssd1306IDisplay<DisplaySize128x64>, screen_data: &ScreenData) {
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
    let mut cursor_txt: String<10> = String::new();
    cursor_txt
        .push_str(format!("{}/{}", selected + 1, screen_data.items.len()).as_str())
        .unwrap();
    Text::with_alignment(
        &cursor_txt,
        bounding_box.bottom_right().unwrap(),
        text_on,
        Alignment::Right,
    )
    .draw(display)
    .unwrap();
    let start_selection = selected.saturating_sub(3);
    let end_selection = screen_data.items.len() as u8;
    for i in start_selection..end_selection {
        let style = if i == *selected {
            Rectangle::new(Point::new(x, y - 1), Size::new(80, 12))
                .into_styled(style_on)
                .draw(display)
                .unwrap();
            text_off
        } else {
            Rectangle::new(Point::new(x, y - 1), Size::new(80, 12))
                .into_styled(style_off)
                .draw(display)
                .unwrap();
            text_on
        };
        Text::with_baseline(
            items[i as usize].as_str(),
            Point::new(x + 2, y),
            style,
            Baseline::Top,
        )
        .draw(display)
        .unwrap();
        y += 12;
    }
}
