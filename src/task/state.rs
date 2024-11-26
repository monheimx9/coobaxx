use crate::task::i2c::I2CDevice;
use crate::task::task_messages::*;
use esp_backtrace as _;
use esp_println as _;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use super::i2c::display::CurrentScreen;
use super::task_messages::EVENT_CHANNEL;

type AppStateManager = Mutex<CriticalSectionRawMutex, Option<AppState>>;
pub static STATE_MANAGER_MUTEX: AppStateManager = Mutex::new(None);

#[derive(Debug)]
pub enum MessagePriority {
    Low,
    Medium,
    High,
}
impl Default for MessagePriority {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug)]
pub enum BroadCastTo {
    Jeremy,
    Mara,
    Teresa,
    All,
}
impl Default for BroadCastTo {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Default)]
pub struct AppState {
    screen: Option<CurrentScreen>,
    message_priority: Option<MessagePriority>,
    broadcast_to: Option<BroadCastTo>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Some(Default::default()),
            message_priority: Some(Default::default()),
            broadcast_to: Some(Default::default()),
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
