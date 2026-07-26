use std::time::SystemTime;

use chrono::{DateTime, Datelike, FixedOffset, Local, Timelike, Utc};

use crate::fs::LocalMinute;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeZoneError;

pub trait TimeZonePort: Send + Sync {
    fn local_minute(&self, instant: SystemTime) -> Result<LocalMinute, TimeZoneError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTimeZone;

impl TimeZonePort for SystemTimeZone {
    fn local_minute(&self, instant: SystemTime) -> Result<LocalMinute, TimeZoneError> {
        let local: DateTime<Local> = instant.into();
        Ok(parts(local))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedTimeZone {
    offset: FixedOffset,
}

impl FixedTimeZone {
    pub fn from_minutes(minutes: i32) -> Result<Self, TimeZoneError> {
        FixedOffset::east_opt(minutes.checked_mul(60).ok_or(TimeZoneError)?)
            .map(|offset| Self { offset })
            .ok_or(TimeZoneError)
    }
}

impl TimeZonePort for FixedTimeZone {
    fn local_minute(&self, instant: SystemTime) -> Result<LocalMinute, TimeZoneError> {
        let utc: DateTime<Utc> = instant.into();
        Ok(parts(utc.with_timezone(&self.offset)))
    }
}

fn parts<Tz: chrono::TimeZone>(value: DateTime<Tz>) -> LocalMinute {
    LocalMinute {
        year: value.year(),
        month: value.month() as u8,
        day: value.day() as u8,
        hour: value.hour() as u8,
        minute: value.minute() as u8,
    }
}
