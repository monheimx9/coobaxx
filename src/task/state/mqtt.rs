use core::str::from_utf8;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use esp_backtrace as _;

use heapless::{String, Vec};

use crate::constant::{MAX_DEVICES, STRING_SIZE};

pub static TOPICS_MUTEX: Mutex<
    CriticalSectionRawMutex,
    Option<Vec<String<STRING_SIZE>, MAX_DEVICES>>,
> = Mutex::new(None);

// pub const CURRENT_DEVICE_NAME: &str = "jeremy";
// Those are also included from the build.rs script
include!("../../broadcast_options.rs");

#[derive(Debug, Clone)]
pub struct MqttSettings {
    number_of_devices: u8,
    broadcast_to: BroadCastTo,
    priotity: MessagePriority,
}
impl Default for MqttSettings {
    fn default() -> Self {
        Self {
            number_of_devices: 0,
            broadcast_to: Default::default(),
            priotity: Default::default(),
        }
    }
}
impl MqttSettings {
    pub fn select_next_recipient(&mut self) {
        self.broadcast_to = self.broadcast_to.next_recipient(self.number_of_devices);
    }
    pub fn selected_recipient(&self) -> u8 {
        match self.broadcast_to {
            BroadCastTo::AllRecipients => self.number_of_devices,
            BroadCastTo::SelectedRecipient(x) => x,
        }
    }
    pub fn set_number_of_devices(&mut self, num: u8) {
        self.number_of_devices = num;
    }
}

#[derive(Debug, Clone)]
pub struct AlertMessage {
    to_device: String<STRING_SIZE>,
    from_device: String<STRING_SIZE>,
    priority: MessagePriority,
}
impl AlertMessage {
    pub fn new_alert(to_device: String<STRING_SIZE>, priority: MessagePriority) -> Self {
        let mut from_device: String<STRING_SIZE> = String::new();
        from_device.push_str(CURRENT_DEVICE_NAME).unwrap();
        AlertMessage {
            to_device,
            from_device,
            priority,
        }
    }
    pub fn payload(&self) -> String<STRING_SIZE> {
        let mut message: String<STRING_SIZE> = String::new();
        message.push_str("to=").unwrap();
        message.push_str(&self.to_device).unwrap();
        message.push_str(";from=").unwrap();
        message.push_str(&self.from_device).unwrap();
        message.push_str(";severity=").unwrap();
        message.push_str(self.priority.as_str()).unwrap();
        message
    }
    pub fn try_new(payload: &[u8]) -> Option<Self> {
        //to=jeremy;from=mara;severity=low
        let msg = from_utf8(payload).ok()?;
        let params = Self::get_params(msg)?;

        if !params.iter().all(|p| p.is_some()) {
            None
        } else {
            Some(AlertMessage {
                to_device: params[0].clone().unwrap().1.clone(),
                from_device: params[1].clone().unwrap().1.clone(),
                priority: MessagePriority::new_from_str(params[2].clone().unwrap().1.as_str()),
            })
        }
    }
    fn get_params(msg: &str) -> Option<Vec<Option<(String<STRING_SIZE>, String<STRING_SIZE>)>, 3>> {
        Some(
            msg.split(';')
                .map(|s| -> Option<(String<STRING_SIZE>, String<STRING_SIZE>)> {
                    let mut k: String<STRING_SIZE> = String::new();
                    let mut v: String<STRING_SIZE> = String::new();
                    let (k_1, v_1) = s.split_once('=')?;
                    k.push_str(k_1).ok()?;
                    v.push_str(v_1).ok()?;
                    Some((k, v))
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub enum PingType {
    Ping,
    Pong,
    Death,
}
impl PingType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pong => "pong=",
            Self::Ping => "ping=",
            Self::Death => "death=",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PingMessage {
    pub ping_type: PingType,
    pub device_name: String<STRING_SIZE>,
}
impl PingMessage {
    pub fn new_ping(ping_type: PingType) -> Self {
        let mut device_name: String<STRING_SIZE> = String::new();
        device_name.push_str(CURRENT_DEVICE_NAME).unwrap();
        PingMessage {
            ping_type,
            device_name,
        }
    }
    pub fn payload(&self) -> String<STRING_SIZE> {
        let mut payload: String<STRING_SIZE> = String::new();
        payload.push_str(self.ping_type.as_str()).unwrap();
        payload.push_str(CURRENT_DEVICE_NAME).unwrap();
        payload
    }
    ///Return None is the device is Self
    fn try_new(payload: &[u8]) -> Option<Self> {
        let msg = from_utf8(payload).ok()?;
        let (k, v) = msg.split_once('=')?;
        if v == CURRENT_DEVICE_NAME {
            return None;
        };
        let mut device_name: String<STRING_SIZE> = String::new();
        match k {
            "ping" => {
                device_name.push_str(v).ok()?;
                Some(PingMessage {
                    ping_type: PingType::Ping,
                    device_name,
                })
            }
            "pong" => {
                device_name.push_str(v).ok()?;
                Some(PingMessage {
                    ping_type: PingType::Pong,
                    device_name,
                })
            }
            "dead" => {
                device_name.push_str(v).ok()?;
                Some(PingMessage {
                    ping_type: PingType::Death,
                    device_name,
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MqttMessage {
    AlertMessage(AlertMessage),
    PingMessage(PingMessage),
}
impl MqttMessage {
    pub fn try_new(topic: &str, payload: &[u8]) -> Option<Self> {
        match topic {
            "magneporc/devices/inbox" => {
                AlertMessage::try_new(payload).map(MqttMessage::AlertMessage)
            }

            "magneporc/devices/ping" => PingMessage::try_new(payload).map(MqttMessage::PingMessage),
            _ => None,
        }
    }
    pub fn topic(&self) -> &str {
        match self {
            Self::AlertMessage(_) => "magneporc/devices/inbox",
            Self::PingMessage(_) => "magneporc/devices/ping",
        }
    }
    pub fn payload(&self) -> String<STRING_SIZE> {
        match self {
            Self::PingMessage(x) => x.payload(),
            Self::AlertMessage(x) => x.payload(),
        }
    }
}
#[derive(Debug, Clone, Copy)]
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
impl MessagePriority {
    pub fn new_from_str(pri: &str) -> Self {
        match pri {
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            _ => Default::default(),
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BroadCastTo {
    SelectedRecipient(u8),
    AllRecipients,
}
impl Default for BroadCastTo {
    fn default() -> Self {
        Self::SelectedRecipient(0)
    }
}
impl BroadCastTo {
    fn next_recipient(self, devices: u8) -> Self {
        match self {
            BroadCastTo::AllRecipients => BroadCastTo::SelectedRecipient(0),
            BroadCastTo::SelectedRecipient(r) => {
                if r >= devices - 1 {
                    BroadCastTo::AllRecipients
                } else {
                    defmt::info!("New recipient={}", r + 1);
                    BroadCastTo::SelectedRecipient(r + 1)
                }
            }
        }
    }
}
