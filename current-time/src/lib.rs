#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use bitflags::bitflags;
use chrono::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayOfWeek {
    Unknown = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl From<u8> for DayOfWeek {
    fn from(value: u8) -> Self {
        match value {
            1 => DayOfWeek::Monday,
            2 => DayOfWeek::Tuesday,
            3 => DayOfWeek::Wednesday,
            4 => DayOfWeek::Thursday,
            5 => DayOfWeek::Friday,
            6 => DayOfWeek::Saturday,
            7 => DayOfWeek::Sunday,
            _ => DayOfWeek::Unknown,
        }
    }
}

impl From<DayOfWeek> for u8 {
    fn from(day: DayOfWeek) -> u8 {
        day as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjustReason(u8);

bitflags! {
    impl AdjustReason: u8 {
        const MANUAL_TIME_UPDATE     = 0x01;
        const EXTERNAL_REFERENCE     = 0x02;
        const TIMEZONE_CHANGE        = 0x04;
        const DST_CHANGE             = 0x08;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub day_of_week: DayOfWeek,
    pub fractions256: u8,
    pub adjust_reason: AdjustReason,
}

impl CurrentTime {

    pub const fn unix_epoch() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            day_of_week: DayOfWeek::Thursday,
            fractions256: 0,
            adjust_reason: AdjustReason::empty(),
        }
    }

    pub fn from_naivedatetime(dt: NaiveDateTime) -> Self {
        Self {
            year: dt.year() as u16,
            month: dt.month() as u8,
            day: dt.day() as u8,
            hour: dt.hour() as u8,
            minute: dt.minute() as u8,
            second: dt.second() as u8,
            day_of_week: DayOfWeek::from(dt.weekday().number_from_monday() as u8),  
            fractions256: 0,
            adjust_reason: AdjustReason::empty(),
        }
    }

    pub fn to_naivedatetime(&self) -> NaiveDateTime {
        let date = NaiveDate::from_ymd_opt(self.year as i32, self.month as u32, self.day as u32).expect("Unable to create date");
        let time = NaiveTime::from_hms_opt(self.hour as u32, self.minute as u32, self.second as u32).expect("Unable to create time");

        NaiveDateTime::new(date, time)
    }
    
    pub fn to_bytes(&self) -> [u8; 10] {
        let year_bytes = self.year.to_le_bytes();
        [
            year_bytes[0],
            year_bytes[1],
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.day_of_week.into(),
            self.fractions256,
            self.adjust_reason.bits(),
        ]
    }

    pub fn from_bytes(bytes: &[u8; 10]) -> Self {
        let year = u16::from_le_bytes([bytes[0], bytes[1]]);
        let month = bytes[2];
        let day = bytes[3];
        let hour = bytes[4];
        let minute = bytes[5];
        let second = bytes[6];
        let day_of_week = DayOfWeek::from(bytes[7]);
        let fractions256 = bytes[8];
        let adjust_reason = AdjustReason::from_bits_truncate(bytes[9]);

        CurrentTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            day_of_week,
            fractions256,
            adjust_reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let ct = CurrentTime {
            year: 2025,
            month: 6,
            day: 9,
            hour: 15,
            minute: 42,
            second: 30,
            day_of_week: DayOfWeek::Monday,
            fractions256: 0,
            adjust_reason: AdjustReason::MANUAL_TIME_UPDATE,
        };

        let bytes = ct.to_bytes();
        let decoded = CurrentTime::from_bytes(&bytes);
        assert_eq!(ct, decoded);
    }

    #[test]
    fn test_encoding_matches_expected_bytes() {
        let ct = CurrentTime {
            year: 2025,
            month: 6,
            day: 9,
            hour: 15,
            minute: 42,
            second: 30,
            day_of_week: DayOfWeek::Monday,
            fractions256: 0,
            adjust_reason: AdjustReason::MANUAL_TIME_UPDATE,
        };

        let expected: [u8; 10] = [0xE9, 0x07, 0x06, 0x09, 0x0F, 0x2A, 0x1E, 0x01, 0x00, 0x01];
        assert_eq!(ct.to_bytes(), expected);
    }

    #[test]
    fn test_adjust_reason_multiple_flags() {
        let ct = CurrentTime {
            year: 2025,
            month: 12,
            day: 31,
            hour: 23,
            minute: 59,
            second: 59,
            day_of_week: DayOfWeek::Sunday,
            fractions256: 255,
            adjust_reason: AdjustReason::MANUAL_TIME_UPDATE
                | AdjustReason::DST_CHANGE
                | AdjustReason::EXTERNAL_REFERENCE,
        };

        let bytes = ct.to_bytes();
        assert_eq!(bytes[9], 0x0B); // 0000_1011

        let decoded = CurrentTime::from_bytes(&bytes);
        assert!(decoded
            .adjust_reason
            .contains(AdjustReason::MANUAL_TIME_UPDATE));
        assert!(decoded.adjust_reason.contains(AdjustReason::DST_CHANGE));
        assert!(decoded
            .adjust_reason
            .contains(AdjustReason::EXTERNAL_REFERENCE));
        assert!(!decoded
            .adjust_reason
            .contains(AdjustReason::TIMEZONE_CHANGE));
    }

    #[test]
    fn test_day_of_week_unknown_handling() {
        let mut bytes = [0u8; 10];
        bytes[7] = 42; // invalid day
        let decoded = CurrentTime::from_bytes(&bytes);
        assert_eq!(decoded.day_of_week, DayOfWeek::Unknown);
    }
}
