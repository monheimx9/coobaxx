use crate::task::state::*;
use crate::task::task_messages::*;

use embassy_time::Timer;
use esp_backtrace as _;
use esp_println as _;

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
        defmt::info!("Event received !");

        '_state_manager_mutex: {
            let mut state_manager_guard = STATE_MANAGER_MUTEX.lock().await;
            let state_manager = state_manager_guard.as_mut().unwrap();

            match event {
                Events::SwitchBroadCastDevice => {
                    defmt::info!("ClearBtn event");

                    state_manager.handle_clr_button().await;
                }
                Events::Standby => {
                    defmt::dbg!("In Standby");
                }
                Events::BroadcastMqtt => {
                    defmt::info!("Waking up");
                    SCHEDULER_START_SIGNAL.signal(Commands::WakeUp);
                }
            }
            drop(state_manager_guard);
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

    loop {
        Timer::after_secs(10).await;
    }
}
