use core::cell::RefCell;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use super::display::CurrentScreen;

static APPSTATE: Mutex<CriticalSectionRawMutex, RefCell<AppState>> =
    Mutex::new(RefCell::new(AppState {
        screen: Some(CurrentScreen::Home),
    }));

#[derive(Debug, Default)]
pub struct AppState {
    pub screen: Option<CurrentScreen>,
}
impl AppState {
    pub fn change_screen(&mut self) {
        self.screen = Some(CurrentScreen::Home);
    }
}
