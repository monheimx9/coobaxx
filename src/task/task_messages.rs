use defmt::Format;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use esp_backtrace as _;

use super::i2c::I2CDevice;
use super::state::MqttMessage;

#[derive(Debug, Clone)]
pub enum Events {
    Starting,
    SelectButtonPressed,
    SendButtonPressed,
    MessageReceived(MqttMessage),
}

#[derive(PartialEq, Debug, Format)]
pub enum Commands {
    UpdateDisplay,
    MqttReceive,
    MqttSend,
    WakeUp,
}

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Events, 10> = Channel::new();
pub static MQTT_SIGNAL_SEND: Signal<CriticalSectionRawMutex, MqttMessage> = Signal::new();
pub static MQTT_SIGNAL_BROKER_PING: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static MQTT_RESET_PING: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub static I2C_MANAGER_SIGNAL: Signal<CriticalSectionRawMutex, I2CDevice> = Signal::new();

pub static SCHEDULER_STOP_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static SCHEDULER_START_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
