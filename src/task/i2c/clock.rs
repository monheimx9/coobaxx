use ds323x::DateTimeAccess;
use ds323x::Datelike;
use ds323x::Ds323x;
use ds323x::NaiveDate;
use ds323x::Timelike;
use esp_hal::i2c::master::AnyI2c;
use esp_hal::i2c::master::I2c;
use esp_hal::Async;
use esp_hal::Blocking;

pub async fn time_rw(i2c: &mut I2c<'static, Async, AnyI2c>) {
    let mut rtc = Ds323x::new_ds3231(i2c);
    let t = rtc.datetime().unwrap();
    let day = t.day();
    let m = t.month();
    let y = t.year();
    let h = t.hour();
    let mm = t.minute();
    let ss = t.second();
    defmt::info!("{}-{}-{} / {}:{}:{}", day, m, y, h, mm, ss);
}
