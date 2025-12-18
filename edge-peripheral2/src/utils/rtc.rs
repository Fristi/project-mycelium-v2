use chrono::NaiveDateTime;

pub trait RtcExt {
    fn now_naivedatetime(&self) -> NaiveDateTime;
}

impl <'a> RtcExt for esp_hal::rtc_cntl::Rtc<'a> {
    fn now_naivedatetime(&self) -> NaiveDateTime {
        let now_us = self.current_time_us() as i64;
        let secs = now_us / 1_000_000;
        let nsecs = (now_us % 1_000_000) * 1_000;

        return NaiveDateTime::from_timestamp(secs, nsecs as u32);
    }
}