#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use bitflags::bitflags;
use chrono::prelude::*;
use timeseries::Deviate;

// BLE Address Service (custom service)
pub const ADDRESS_SERVICE_UUID_16: u16 = 0xFFF5;
pub const ADDRESS_CHARACTERISTIC_UUID_16: u16 = 0xFFF7;

// BLE Measurement Service (custom service)
pub const MEASUREMENT_SERVICE_UUID_16: u16 = 0xFFF6;
pub const MEASUREMENT_CHARACTERISTIC_UUID_16: u16 =  0xFFF8;

// BLE Current Time Service (standard BLE service)
pub const CURRENT_TIME_SERVICE_UUID: u16 = 0x1805;
pub const CURRENT_TIME_CHARACTERISTIC_UUID: u16 = 0x2a2b;


#[derive(Clone, Copy)]
pub struct MeasurementSerieEntry {
    pub timestamp: NaiveDateTime,
    pub measurement: Measurement
}

impl MeasurementSerieEntry {
    /// Encode measurement series entry to TLV format
    pub fn to_tlv(&self) -> [u8; 39] {
        let mut tlv = [0u8; 39];
        let mut index = 0;
        
        // Timestamp (Type: 1) - 8 bytes for i64 timestamp
        tlv[index] = 1; // Type
        index += 1;
        tlv[index] = 8; // Length
        index += 1;
        let timestamp_bytes = self.timestamp.and_utc().timestamp().to_le_bytes();
        tlv[index..index + 8].copy_from_slice(&timestamp_bytes);
        index += 8;
        
        // Measurement (Type: 2) - 21 bytes for measurement
        tlv[index] = 2; // Type
        index += 1;
        tlv[index] = 27; // Length
        index += 1;
        tlv[index..index + 27].copy_from_slice(&self.measurement.to_tlv());
        
        tlv
    }
    
    /// Decode measurement series entry from TLV format
    pub fn from_tlv(data: &[u8]) -> Result<Self, &'static str> {
        let mut index = 0;
        let mut timestamp = None;
        let mut measurement = None;
        
        while index < data.len() {
            if index + 1 >= data.len() {
                return Err("Incomplete TLV data");
            }
            
            let tlv_type = data[index];
            index += 1;
            let length = data[index] as usize;
            index += 1;
            
            if index + length > data.len() {
                return Err("TLV length exceeds data bounds");
            }
            
            let value = &data[index..index + length];
            
            match tlv_type {
                1 => { // Timestamp
                    if length != 8 {
                        return Err("Invalid timestamp length");
                    }
                    let timestamp_i64 = i64::from_le_bytes([
                        value[0], value[1], value[2], value[3],
                        value[4], value[5], value[6], value[7]
                    ]);
                    timestamp = Some(DateTime::from_timestamp(timestamp_i64, 0)
                        .ok_or("Invalid timestamp")?.naive_utc());
                }
                2 => { // Measurement
                    if length != 21 {
                        return Err("Invalid measurement length");
                    }
                    measurement = Some(Measurement::from_tlv(value)?);
                }
                _ => {
                    return Err("Unknown TLV type");
                }
            }
            
            index += length;
        }
        
        let timestamp = timestamp.ok_or("Missing timestamp")?;
        let measurement = measurement.ok_or("Missing measurement")?;
        
        Ok(MeasurementSerieEntry {
            timestamp,
            measurement
        })
    }
}



#[derive(Clone, Copy)]
pub struct Measurement {
    pub battery: u8,
    pub lux: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub soil_pf: f32
}

impl Deviate for Measurement {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        (self.temperature - other.temperature).abs() > max_deviation.temperature ||
        (self.humidity - other.humidity).abs() > max_deviation.humidity ||
        self.battery.abs_diff(other.battery) > max_deviation.battery ||
        (self.lux - other.lux).abs() > max_deviation.lux
    }
}

impl Measurement {
    pub const MAX_DEVIATION: Self = Self {
        battery: 1,
        lux: 100.0,
        temperature: 1.0,
        humidity: 0.1,
        soil_pf: 0.1
    };
}

impl Measurement {
    /// Encode measurement to TLV format
    pub fn to_tlv(&self) -> [u8; 27] {
        let mut tlv = [0u8; 27];
        let mut index = 0;
        
        // Battery (Type: 1)
        tlv[index] = 1; // Type
        index += 1;
        tlv[index] = 1; // Length
        index += 1;
        tlv[index] = self.battery;
        index += 1;
        
        // Lux (Type: 2)
        tlv[index] = 2; // Type
        index += 1;
        tlv[index] = 4; // Length
        index += 1;
        tlv[index..index + 4].copy_from_slice(&self.lux.to_le_bytes());
        index += 4;
        
        // Temperature (Type: 3)
        tlv[index] = 3; // Type
        index += 1;
        tlv[index] = 4; // Length
        index += 1;
        tlv[index..index + 4].copy_from_slice(&self.temperature.to_le_bytes());
        index += 4;
        
        // Humidity (Type: 4)
        tlv[index] = 4; // Type
        index += 1;
        tlv[index] = 4; // Length
        index += 1;
        tlv[index..index + 4].copy_from_slice(&self.humidity.to_le_bytes());
        index += 4;

        // Soil_pf (Type: 5)
        tlv[index] = 5; // Type
        index += 1;
        tlv[index] = 4; // Length
        index += 1;
        tlv[index..index + 4].copy_from_slice(&self.soil_pf.to_le_bytes());
        
        tlv
    }
    
    /// Decode measurement from TLV format
    pub fn from_tlv(data: &[u8]) -> Result<Self, &'static str> {
        let mut measurement = Measurement {
            battery: 0,
            lux: 0.0,
            temperature: 0.0,
            humidity: 0.0,
            soil_pf: 0.0
        };
        
        let mut i = 0;
        while i < data.len() {
            if i + 2 > data.len() {
                return Err("Incomplete TLV header");
            }
            
            let tlv_type = data[i];
            let length = data[i + 1] as usize;
            
            if i + 2 + length > data.len() {
                return Err("Incomplete TLV data");
            }
            
            let value_data = &data[i + 2..i + 2 + length];
            
            match tlv_type {
                1 => { // Battery
                    if length != 1 {
                        return Err("Invalid battery length");
                    }
                    measurement.battery = value_data[0];
                },
                2 => { // Lux
                    if length != 4 {
                        return Err("Invalid lux length");
                    }
                    measurement.lux = f32::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3]]);
                },
                3 => { // Temperature
                    if length != 4 {
                        return Err("Invalid temperature length");
                    }
                    measurement.temperature = f32::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3]]);
                },
                4 => { // Humidity
                    if length != 4 {
                        return Err("Invalid humidity length");
                    }
                    measurement.humidity = f32::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3]]);
                },
                5 => { // Soil_pf
                    if length != 4 {
                        return Err("Invalid soil pf length");
                    }
                    measurement.soil_pf = f32::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3]]);
                },
                _ => return Err("Unknown TLV type"),
            }
            
            i += 2 + length;
        }
        
        Ok(measurement)
    }
}





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

    pub fn from_bytes(bytes: &[u8]) -> Self {
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

    #[test]
    fn test_measurement_tlv_encode_decode() {
        let measurement = Measurement {
            battery: 85,
            lux: 1234.56,
            temperature: 23.5,
            humidity: 45.2,
        };
        
        let tlv_data = measurement.to_tlv();
        let decoded = Measurement::from_tlv(&tlv_data).unwrap();
        
        assert_eq!(measurement.battery, decoded.battery);
        assert!((measurement.lux - decoded.lux).abs() < f32::EPSILON);
        assert!((measurement.temperature - decoded.temperature).abs() < f32::EPSILON);
        assert!((measurement.humidity - decoded.humidity).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_measurement_tlv_invalid_data() {
        // Test with incomplete data
        let result = Measurement::from_tlv(&[1, 1]); // Missing value
        assert!(result.is_err());
        
        // Test with unknown type
        let result = Measurement::from_tlv(&[99, 1, 0]); // Unknown type 99
        assert!(result.is_err());
        
        // Test with wrong length for battery
        let result = Measurement::from_tlv(&[1, 2, 0, 0]); // Battery with length 2
        assert!(result.is_err());
    }
    
    #[test]
    fn test_measurement_tlv_empty() {
        let measurement = Measurement {
            battery: 0,
            lux: 0.0,
            temperature: 0.0,
            humidity: 0.0,
        };
        
        let tlv_data = measurement.to_tlv();
        let decoded = Measurement::from_tlv(&tlv_data).unwrap();
        
        assert_eq!(measurement.battery, decoded.battery);
        assert_eq!(measurement.lux, decoded.lux);
        assert_eq!(measurement.temperature, decoded.temperature);
        assert_eq!(measurement.humidity, decoded.humidity);
    }

    #[test]
    fn test_measurement_serie_entry_tlv_encode_decode() {
        let measurement = Measurement {
            battery: 75,
            lux: 987.65,
            temperature: 18.3,
            humidity: 62.1,
        };
        
        let entry = MeasurementSerieEntry {
            timestamp: DateTime::from_timestamp(1640995200, 0).unwrap().naive_utc(), // 2022-01-01 00:00:00
            measurement,
        };
        
        let tlv_data = entry.to_tlv();
        let decoded = MeasurementSerieEntry::from_tlv(&tlv_data).unwrap();
        
        assert_eq!(entry.timestamp, decoded.timestamp);
        assert_eq!(entry.measurement.battery, decoded.measurement.battery);
        assert!((entry.measurement.lux - decoded.measurement.lux).abs() < f32::EPSILON);
        assert!((entry.measurement.temperature - decoded.measurement.temperature).abs() < f32::EPSILON);
        assert!((entry.measurement.humidity - decoded.measurement.humidity).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_measurement_serie_entry_tlv_invalid_data() {
        // Test with incomplete data
        let result = MeasurementSerieEntry::from_tlv(&[1, 8]); // Missing timestamp value
        assert!(result.is_err());
        
        // Test with missing measurement
        let mut incomplete_data = [0u8; 10];
        incomplete_data[0] = 1; // Type: timestamp
        incomplete_data[1] = 8; // Length: 8
        // Timestamp bytes would go here, but we're testing missing measurement
        let result = MeasurementSerieEntry::from_tlv(&incomplete_data);
        assert!(result.is_err());
        
        // Test with unknown TLV type
        let result = MeasurementSerieEntry::from_tlv(&[99, 1, 0]); // Unknown type 99
        assert!(result.is_err());
        
        // Test with wrong timestamp length
        let result = MeasurementSerieEntry::from_tlv(&[1, 4, 0, 0, 0, 0]); // Timestamp with length 4
        assert!(result.is_err());
    }
    
    #[test]
    fn test_measurement_serie_entry_tlv_empty_values() {
        let measurement = Measurement {
            battery: 0,
            lux: 0.0,
            temperature: 0.0,
            humidity: 0.0,
        };
        
        let entry = MeasurementSerieEntry {
            timestamp: DateTime::from_timestamp(0, 0).unwrap().naive_utc(), // Unix epoch
            measurement,
        };
        
        let tlv_data = entry.to_tlv();
        let decoded = MeasurementSerieEntry::from_tlv(&tlv_data).unwrap();
        
        assert_eq!(entry.timestamp, decoded.timestamp);
        assert_eq!(entry.measurement.battery, decoded.measurement.battery);
        assert_eq!(entry.measurement.lux, decoded.measurement.lux);
        assert_eq!(entry.measurement.temperature, decoded.measurement.temperature);
        assert_eq!(entry.measurement.humidity, decoded.measurement.humidity);
    }

}