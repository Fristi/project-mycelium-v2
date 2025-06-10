#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use bitflags::bitflags;

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
    
    pub fn from_unix_timestamp(secs: i64) -> Option<Self> {
        // Constants for time calculations
        const SECS_PER_DAY: i64 = 86400;
        const SECS_PER_HOUR: i64 = 3600;
        const SECS_PER_MINUTE: i64 = 60;
        
        // Days since epoch
        let days_since_epoch = secs / SECS_PER_DAY;
        let seconds_in_day = secs % SECS_PER_DAY;
        
        // Calculate time components
        let hour = (seconds_in_day / SECS_PER_HOUR) as u8;
        let minute = ((seconds_in_day % SECS_PER_HOUR) / SECS_PER_MINUTE) as u8;
        let second = (seconds_in_day % SECS_PER_MINUTE) as u8;
        
        // Calculate date components
        let mut year = 1970;
        let mut days_remaining = days_since_epoch;
        
        // Find year
        while days_remaining > 0 {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if days_remaining < days_in_year {
                break;
            }
            days_remaining -= days_in_year;
            year += 1;
        }
        
        // Find month and day
        let mut month = 1;
        let mut day = 1;
        while days_remaining > 0 {
            let days_in_month = match month {
                2 => if is_leap_year(year) { 29 } else { 28 },
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            if days_remaining < days_in_month {
                day = days_remaining as u8 + 1;
                break;
            }
            days_remaining -= days_in_month;
            month += 1;
        }
        
        // Calculate day of week (1970-01-01 was a Thursday)
        let day_of_week = match (days_since_epoch % 7) as u8 {
            0 => DayOfWeek::Thursday,
            1 => DayOfWeek::Friday,
            2 => DayOfWeek::Saturday,
            3 => DayOfWeek::Sunday,
            4 => DayOfWeek::Monday,
            5 => DayOfWeek::Tuesday,
            6 => DayOfWeek::Wednesday,
            _ => DayOfWeek::Unknown,
        };
        
        Some(CurrentTime {
            year: year as u16,
            month: month as u8,
            day,
            hour,
            minute,
            second,
            day_of_week,
            fractions256: 0,
            adjust_reason: AdjustReason::empty(),
        })
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

    /// Returns (seconds since UNIX epoch, nanoseconds)
    /// Assumes UTC time. Ignores leap seconds.
    pub fn to_unix_timestamp(&self) -> Option<(i64, u32)> {
        // Reject invalid months or days
        if self.month < 1 || self.month > 12 || self.day < 1 || self.day > 31 {
            return None;
        }

        let days_in_month = |year: u16, month: u8| -> u8 {
            match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    if is_leap_year(year) {
                        29
                    } else {
                        28
                    }
                }
                _ => 0,
            }
        };

        if self.day > days_in_month(self.year, self.month) {
            return None;
        }

        let days_since_epoch = date_to_days_since_epoch(self.year, self.month, self.day)?;
        let seconds = days_since_epoch as i64 * 86400
            + self.hour as i64 * 3600
            + self.minute as i64 * 60
            + self.second as i64;

        let nano_fraction = self.fractions256 as f32 / 256_f32;
        let nanos = nano_fraction * 1_000_000_000f32;

        Some((seconds, nanos as u32))
    }
}

/// Returns true if year is a leap year
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Converts a date to the number of days since UNIX epoch (1970-01-01)
fn date_to_days_since_epoch(year: u16, month: u8, day: u8) -> Option<u32> {
    // UNIX epoch starts at 1970-01-01
    if year < 1970 || month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }

    // Count days in full years since 1970
    let mut days = 0u32;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Count days in full months of current year
    let month_lengths = [
        31,                                       // Jan
        if is_leap_year(year) { 29 } else { 28 }, // Feb
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    for m in 0..(month as usize - 1) {
        days += month_lengths[m] as u32;
    }

    // Add current day (1-based)
    days += (day - 1) as u32;

    Some(days)
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

    #[test]
    fn test_unix_timestamp_basic() {
        let ct = CurrentTime {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            day_of_week: DayOfWeek::Thursday,
            fractions256: 0,
            adjust_reason: AdjustReason::empty(),
        };

        let (secs, nanos) = ct.to_unix_timestamp().unwrap();
        assert_eq!(secs, 0);
        assert_eq!(nanos, 0);
    }

    #[test]
    fn test_unix_timestamp_half_second() {
        let ct = CurrentTime {
            year: 2025,
            month: 6,
            day: 9,
            hour: 15,
            minute: 42,
            second: 30,
            day_of_week: DayOfWeek::Monday,
            fractions256: 128, // 0.5 seconds
            adjust_reason: AdjustReason::MANUAL_TIME_UPDATE,
        };

        let (secs, nanos) = ct.to_unix_timestamp().unwrap();
        assert_eq!(secs, 1749483750);
        assert_eq!(nanos, 500_000_000);
    }

    #[test]
    fn test_unix_timestamp_invalid_date() {
        let ct = CurrentTime {
            year: 2025,
            month: 2,
            day: 30, // invalid date
            hour: 0,
            minute: 0,
            second: 0,
            day_of_week: DayOfWeek::Unknown,
            fractions256: 0,
            adjust_reason: AdjustReason::empty(),
        };

        assert_eq!(ct.to_unix_timestamp(), None);
    }

    #[test]
    fn test_leap_year_feb_29() {
        let ct = CurrentTime {
            year: 2024,
            month: 2,
            day: 29,
            hour: 12,
            minute: 0,
            second: 0,
            day_of_week: DayOfWeek::Thursday,
            fractions256: 0,
            adjust_reason: AdjustReason::empty(),
        };

        let (secs, _) = ct.to_unix_timestamp().unwrap();
        assert_eq!(secs, 1709208000); // Confirmed using UNIX tools
    }

    #[test]
    fn test_from_unix_timestamp_basic() {
        let ct = CurrentTime::from_unix_timestamp(1749483750).unwrap();
        assert_eq!(ct.year, 2025);
        assert_eq!(ct.month, 6);
        assert_eq!(ct.day, 9);
        assert_eq!(ct.hour, 15);
        assert_eq!(ct.minute, 42);
        assert_eq!(ct.second, 30);
        assert_eq!(ct.day_of_week, DayOfWeek::Monday);
        assert_eq!(ct.fractions256, 0);
        assert_eq!(ct.adjust_reason, AdjustReason::empty());
    }

    #[test]
    fn test_from_unix_timestamp_epoch() {
        let ct = CurrentTime::from_unix_timestamp(0).unwrap();
        assert_eq!(ct.year, 1970);
        assert_eq!(ct.month, 1);
        assert_eq!(ct.day, 1);
        assert_eq!(ct.hour, 0);
        assert_eq!(ct.minute, 0);
        assert_eq!(ct.second, 0);
        assert_eq!(ct.day_of_week, DayOfWeek::Thursday);
        assert_eq!(ct.fractions256, 0);
        assert_eq!(ct.adjust_reason, AdjustReason::empty());
    }

    #[test]
    fn test_from_unix_timestamp_leap_year() {
        let ct = CurrentTime::from_unix_timestamp(1709208000).unwrap();
        assert_eq!(ct.year, 2024);
        assert_eq!(ct.month, 2);
        assert_eq!(ct.day, 29);
        assert_eq!(ct.hour, 12);
        assert_eq!(ct.minute, 0);
        assert_eq!(ct.second, 0);
        assert_eq!(ct.day_of_week, DayOfWeek::Thursday);
        assert_eq!(ct.fractions256, 0);
        assert_eq!(ct.adjust_reason, AdjustReason::empty());
    }
}
