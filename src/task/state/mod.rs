use core::str::FromStr;

use crate::task::i2c::I2CDevice;
use crate::task::task_messages::*;
use esp_backtrace as _;
use esp_println as _;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use heapless::{String, Vec};

use super::i2c::display::CurrentScreen;
use super::task_messages::EVENT_CHANNEL;

mod logic;
use logic::*;

mod mqtt;
pub use mqtt::*;

use crate::constant::{MAX_DEVICES, STRING_SIZE};

type AppStateManager = Mutex<CriticalSectionRawMutex, Option<AppState>>;
pub static STATE_MANAGER_MUTEX: AppStateManager = Mutex::new(None);
pub static SCREEN_STATE_MUTEX: Mutex<CriticalSectionRawMutex, Option<ScreenData>> =
    Mutex::new(None);
pub const GLOBAL_MAX_ITEMS: usize = 12;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    screen: Option<CurrentScreen>,
    mqtt_settings: Option<MqttSettings>,
    mqtt_msg: Option<MqttMessage>,
    devices: Option<Vec<String<STRING_SIZE>, MAX_DEVICES>>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Some(Default::default()),
            mqtt_settings: Some(Default::default()),
            mqtt_msg: None,
            devices: None,
        }
    }
}

///State transitions
impl AppState {
    pub async fn wake_up(&mut self) {
        todo!()
    }
    pub async fn change_screen(&mut self, new_s: CurrentScreen) {
        self.screen = Some(new_s);
        I2C_MANAGER_SIGNAL.signal(I2CDevice::Ssd1306Display);
    }
    pub async fn prepare_mqtt_msg(&mut self) {
        let dev_name = self.selected_device_name();
        let pri = MessagePriority::Medium;
        let alert = AlertMessage::new_alert(dev_name, pri);
        self.mqtt_msg = Some(MqttMessage::AlertMessage(alert));
    }
    pub async fn clear_alert(&mut self) {
        todo!()
    }
    pub async fn show_error() {
        todo!()
    }
}

///User inputs
impl AppState {
    pub async fn select_btn(&mut self) {
        match self.screen.unwrap_or(CurrentScreen::Messaging) {
            CurrentScreen::Starting => {}
            CurrentScreen::Messaging => {
                let mq = self.mqtt_settings.as_mut().unwrap();
                let num = self.devices.as_ref().map(|f| f.len() as u8).unwrap_or(0_u8);
                mq.set_number_of_devices(num);
                mq.select_next_recipient();
                self.prepare_mqtt_msg().await;
            }
            CurrentScreen::MessageSent => {}
            CurrentScreen::Alert => {
                self.clear_alert().await;
                self.change_screen(CurrentScreen::Messaging).await;
            }
            CurrentScreen::Error => {}
        }
    }
    pub async fn send_btn(&mut self) {
        match self.screen.unwrap_or(CurrentScreen::Messaging) {
            CurrentScreen::Messaging => {
                if let Some(msg) = self.mqtt_msg.as_ref() {
                    MQTT_SIGNAL_SEND.signal(msg.clone());
                }
            }
            _ => {}
        }
    }
    pub async fn receive_msg(&mut self, msg: MqttMessage) {
        match msg {
            MqttMessage::AlertMessage(a) => {}
            MqttMessage::PingMessage(p) => match p.ping_type {
                PingType::Ping => {}
                PingType::Pong => self.add_device(p.device_name),
                PingType::Death => self.remove_device(p.device_name),
            },
        }
    }
}

///Data
impl AppState {
    pub async fn screen_data(&self) {
        let mut items = Vec::new();
        if self.devices.is_some() {
            for i in self.devices.as_ref().unwrap() {
                let _ = items.push(i.clone());
            }
        }
        let mut broadcast_all: String<STRING_SIZE> = String::new();
        broadcast_all.push_str("Tout le monde").unwrap();
        let _ = items.push(broadcast_all);
        let screen_data = ScreenData {
            items,
            selected_item: self
                .mqtt_settings
                .clone()
                .unwrap_or_default()
                .selected_recipient(),
            align: false,
            current_screen: self.screen.unwrap_or(CurrentScreen::Messaging),
        };
        {
            *(SCREEN_STATE_MUTEX.lock().await) = Some(screen_data);
        }
    }
    pub fn add_device(&mut self, device_name: String<STRING_SIZE>) {
        if let Some(dev) = self.devices.as_mut() {
            if !dev.iter().any(|d| d == &device_name) {
                let _ = dev.push(device_name.clone());
            }
        } else {
            let mut dev: Vec<String<STRING_SIZE>, MAX_DEVICES> = Vec::new();
            let _ = dev.push(device_name.clone());
            self.devices = Some(dev);
        }
    }
    pub fn remove_device(&mut self, device_name: String<STRING_SIZE>) {
        if let Some(dev) = self.devices.as_mut() {
            if let Some((index, _)) = dev.iter().enumerate().find(|(a, b)| device_name == **b) {
                dev.remove(index);
            };
        }
    }
}

// pub fn mqtt_settings<'a>(&self) -> Option<&'a MqttSettings> {
//     self.mqtt_msg.as_ref()
// }
// pub async fn set_topics(&mut self) {
//     self.mqtt_msg.as_mut().unwrap().set_topics().await;
// }
// }

pub struct ScreenData {
    pub items: Vec<String<STRING_SIZE>, GLOBAL_MAX_ITEMS>,
    pub selected_item: u8,
    pub align: bool,
    pub current_screen: CurrentScreen,
}
