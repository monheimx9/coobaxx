use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::gpio::{Input, Level};

use super::task_messages::{Events, EVENT_CHANNEL};

pub struct ButtonManager<'a> {
    input: Input<'a>,
    events: Events,
    sender: Sender<'a, CriticalSectionRawMutex, Events, 10>,
    debounce_duration: Duration,
}
impl<'a> ButtonManager<'a> {
    pub fn new(
        input: Input<'a>,
        events: Events,
        sender: Sender<'a, CriticalSectionRawMutex, Events, 10>,
    ) -> Self {
        Self {
            input,
            events,
            sender,
            debounce_duration: Duration::from_millis(80),
        }
    }
    pub async fn debounce(&mut self) -> Level {
        loop {
            let l1 = self.input.level();
            self.input.wait_for_any_edge().await;
            Timer::after(self.debounce_duration).await;
            let l2 = self.input.level();
            if l1 != l2 {
                defmt::info!("Debounced");
                break l2;
            }
        }
    }
    pub async fn handle_button_press(&mut self) {
        'mainloop: loop {
            let init_level = self.debounce().await;
            defmt::info!("Handler button press debounced");
            if init_level == Level::Low {
                continue 'mainloop;
            };
            let event = match self.events {
                Events::SelectButtonPressed => self.events.clone(),
                Events::SendButtonPressed => self.events.clone(),
                _ => panic!("Invalid event"),
            };
            self.sender.send(event).await;
            defmt::info!("Event sended");
        }
    }
}

#[embassy_executor::task]
pub async fn handler_select_btn(ipt: Input<'static>) {
    let sender = EVENT_CHANNEL.sender();
    let mut btn = ButtonManager::new(ipt, Events::SelectButtonPressed, sender);
    defmt::info!("Button handler started");
    btn.handle_button_press().await;
}

#[embassy_executor::task]
pub async fn handler_send_btn(ipt: Input<'static>) {
    let sender = EVENT_CHANNEL.sender();
    let mut btn = ButtonManager::new(ipt, Events::SendButtonPressed, sender);
    defmt::info!("Button handler 2 started");
    btn.handle_button_press().await;
}
