use crate::task::i2c::I2CDevice;
use crate::task::task_messages::*;
use esp_backtrace as _;
use esp_println as _;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use super::i2c::display::CurrentScreen;
use super::mqtt::{MessagePriority, MqttMessage};
use super::task_messages::EVENT_CHANNEL;

type AppStateManager = Mutex<CriticalSectionRawMutex, Option<AppState>>;
pub static STATE_MANAGER_MUTEX: AppStateManager = Mutex::new(None);

#[derive(Debug, Default)]
pub struct AppState {
    screen: Option<CurrentScreen>,
    mqtt_msg: Option<MqttMessage>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Some(Default::default()),
            mqtt_msg: Some(Default::default()),
        }
    }
    pub fn change_screen(&mut self, new_s: CurrentScreen) {
        self.screen = Some(new_s);
        I2C_MANAGER_SIGNAL.signal(I2CDevice::Ssd1306Display(new_s));
        defmt::info!("Change screen signaled");
    }
}

///State transitions
impl AppState {
    pub async fn wake_up(&mut self) {
        defmt::info!("AppState::wake_up");
        let sender = EVENT_CHANNEL.sender();
        sender.send(Events::BroadcastMqtt).await;
    }
}

///User inputs
impl AppState {
    pub async fn handle_clr_button(&mut self) {
        defmt::info!("AppState::handle_clr_button");
        self.change_screen(CurrentScreen::Starting);
        // self.wake_up().await
    }
}
