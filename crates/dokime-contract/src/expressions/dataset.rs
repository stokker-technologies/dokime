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

// Third Party Imports

// Local Imports
use crate::core::{DurationMillis, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordTimingRecord<'a> {
    pub record_type: &'a str,
    pub sequence: u64,
    pub timestamp: &'a Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordTimingPoint {
    pub end_record_type: String,
    pub previous_record_type: String,
    pub start_sequence: u64,
    pub end_sequence: u64,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub duration: DurationMillis,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn record_time_between<'a>(
    records: impl IntoIterator<Item = RecordTimingRecord<'a>>,
    end_record_type: &str,
    previous_record_type: &str,
) -> Vec<RecordTimingPoint> {
    let mut records = records.into_iter().collect::<Vec<_>>();
    records.sort_by_key(|record| record.sequence);

    let mut previous_record: Option<RecordTimingRecord<'_>> = None;
    let mut points = Vec::new();
    for record in records {
        if record.record_type == end_record_type
            && let Some(start) = previous_record
            && let Some(duration) = start.timestamp.millis_until(record.timestamp)
        {
            points.push(RecordTimingPoint {
                end_record_type: end_record_type.to_owned(),
                previous_record_type: previous_record_type.to_owned(),
                start_sequence: start.sequence,
                end_sequence: record.sequence,
                started_at: start.timestamp.clone(),
                ended_at: record.timestamp.clone(),
                duration,
            });
        }

        if record.record_type == previous_record_type {
            previous_record = Some(record);
        }
    }

    points
}

/* === Private Functions === */

/* === Tests === */
