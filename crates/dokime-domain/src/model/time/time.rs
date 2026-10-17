/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::cmp::Ordering;
use std::fmt;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurationMillis(pub u64);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for Timestamp {
    fn memory_size(&self) -> usize {
        harmos::MemorySize::memory_size(&self.0)
    }
}

impl harmos::MemorySize for DurationMillis {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Timestamp {
    pub fn to_datetime(&self) -> Result<time::OffsetDateTime, time::error::Parse> {
        time::OffsetDateTime::parse(&self.0, &time::format_description::well_known::Rfc3339)
            .map(|value| value.to_offset(time::UtcOffset::UTC))
    }

    pub fn from_datetime(value: time::OffsetDateTime) -> Self {
        Self(datetime_text(value))
    }

    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn from_system_time(value: SystemTime) -> Self {
        let unix_nanos = match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos() as i128,
            Err(error) => -(error.duration().as_nanos() as i128),
        };
        Self(format_rfc3339(unix_nanos))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn millis_until(&self, other: &Self) -> Option<DurationMillis> {
        let start = self.parsed()?;
        let end = other.parsed()?;
        u64::try_from((end - start) / 1_000_000)
            .ok()
            .map(DurationMillis)
    }

    pub fn unix_millis(&self) -> Option<u64> {
        u64::try_from(self.parsed()? / 1_000_000).ok()
    }

    fn parsed(&self) -> Option<i128> {
        parse_rfc3339(&self.0)
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.parsed(), other.parsed()) {
            (Some(left), Some(right)) => left
                .cmp(&right)
                // Preserve the `Ord`/`Eq` contract for distinct RFC 3339
                // representations of the same instant.
                .then_with(|| self.0.cmp(&other.0)),
            // Keep malformed legacy values in a deterministic bucket without
            // letting a lexical comparison disturb valid chronological order.
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self.0.cmp(&other.0),
        }
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<Timestamp> for crate::model::values::Value {
    fn from(timestamp: Timestamp) -> Self {
        Self::Datetime(
            timestamp
                .to_datetime()
                .expect("a timestamp carried as a value must be valid RFC3339"),
        )
    }
}

/* === Public Functions === */
pub fn datetime_text(value: time::OffsetDateTime) -> String {
    value
        .to_offset(time::UtcOffset::UTC)
        .format(&time::format_description::well_known::Rfc3339)
        .expect("a stored datetime is representable as RFC3339")
}

/* === Private Functions === */
fn parse_rfc3339(value: &str) -> Option<i128> {
    time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .ok()
        .map(|value| value.unix_timestamp_nanos())
}

fn format_rfc3339(unix_nanos: i128) -> String {
    datetime_text(
        time::OffsetDateTime::from_unix_timestamp_nanos(unix_nanos)
            .expect("a system timestamp is within the supported time range"),
    )
}

/* === Tests === */
