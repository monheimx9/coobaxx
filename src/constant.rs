pub const MAX_DEVICES: usize = 20;
pub const STRING_SIZE: usize = 40;

pub const MQTT_SUB_TOPICS_SIZE: usize = 2;
pub const MQTT_SUB_TOPICS: &[&str; MQTT_SUB_TOPICS_SIZE] =
    &["magneporc/devices/ping", "magneporc/devices/inbox"];
