use crate::task::state::*;
use crate::task::task_messages::*;
use crate::I2CDevice;

use embassy_futures::select::select;
use embassy_futures::select::Either;
use embassy_time::Timer;
use esp_backtrace as _;

#[embassy_executor::task]
pub async fn orchestrator() {
    defmt::info!("orchestrator starting");

    {
        let state_manager = AppState::new();
        *(STATE_MANAGER_MUTEX.lock().await) = Some(state_manager);
    }

    let event_receiver = EVENT_CHANNEL.receiver();

    loop {
        let event = event_receiver.receive().await;
        '_state_manager_mutex: {
            let mut state_manager_guard = STATE_MANAGER_MUTEX.lock().await;
            let state_manager = state_manager_guard.as_mut().unwrap();

            match event {
                Events::Starting => {
                    state_manager.send_mqtt_ping(PingType::Ping).await;
                }
                Events::SelectButtonPressed => {
                    defmt::info!("SelectButtonPressed Event received by orchestrator");
                    state_manager.select_btn().await;
                }
                Events::SendButtonPressed => {
                    state_manager.send_btn().await;
                    // MQTT_SIGNAL_SEND.signal(());
                    SCHEDULER_START_SIGNAL.signal(());
                }
                Events::MessageReceived(mqtt_msg) => state_manager.receive_msg(mqtt_msg).await,
            }
            state_manager.screen_data().await;
            drop(state_manager_guard);
            I2C_MANAGER_SIGNAL.signal(I2CDevice::Ssd1306Display);
        }
    }
}

#[embassy_executor::task]
pub async fn scheduler() {
    defmt::info!("Scheduler starting");
    if SCHEDULER_STOP_SIGNAL.signaled() {
        SCHEDULER_STOP_SIGNAL.reset();
        SCHEDULER_START_SIGNAL.wait().await;
    }

    EVENT_CHANNEL.send(Events::Starting).await;

    loop {
        // MQTT_SIGNAL_RECEIVE.signal(());
        // MQTT_SIGNAL_SEND.signal(());
        // EVENT_CHANNEL.send(Events::SelectButtonPressed).await;
        match select(MQTT_RESET_PING.wait(), Timer::after_secs(50)).await {
            Either::First(_) => {
                MQTT_RESET_PING.reset();
                I2C_MANAGER_SIGNAL.signal(I2CDevice::RtcDs3231(crate::RtcAction::Read));
            }
            Either::Second(_) => {
                MQTT_SIGNAL_BROKER_PING.signal(());
            }
        }
    }
}
