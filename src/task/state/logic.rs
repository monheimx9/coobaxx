use heapless::String;

use super::AppState;
use super::STRING_SIZE;

//Helper functions
impl AppState {
    pub fn selected_device_name(&self) -> String<STRING_SIZE> {
        let num = self
            .mqtt_settings
            .as_ref()
            .map(|f| f.selected_recipient())
            .unwrap_or(0);
        let dev: Option<&String<STRING_SIZE>> =
            self.devices.as_ref().and_then(|f| f.get(num as usize));
        if let Some(a) = dev {
            a.clone()
        } else {
            let mut x: String<STRING_SIZE> = String::new();
            x.push_str("all").unwrap();
            x
        }
    }
}
