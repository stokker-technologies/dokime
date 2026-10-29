/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Disk-Spool Volume Fixture

use super::*;

pub fn capture(days: u64, per_day: u64) {
    let mut rows = EvidenceRows::<(u64, String)>::default();
    let mut previous_bytes = 0;
    for day in 0..days {
        for minute in 0..per_day {
            rows.append(&(day * per_day + minute, "x".repeat(256))).unwrap();
        }
        assert_eq!(rows.len(), ((day + 1) * per_day) as usize);
        assert_eq!(rows.retained_rows(), 0);
        assert!(rows.spooled_bytes() > previous_bytes);
        previous_bytes = rows.spooled_bytes();
    }
    let retained = rows.clone();
    drop(rows);
    assert_eq!(retained.retained_rows(), 0);
    for (index, row) in retained.iter().enumerate() {
        assert_eq!(row.0, index as u64);
        assert_eq!(row.1.len(), 256);
    }
}
