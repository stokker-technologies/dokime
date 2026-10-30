/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(unix)]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

/* === Re-Exports === */

#[cfg(unix)]
pub use unix::Notifications;
#[cfg(target_os = "windows")]
pub use windows::Notifications;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Change {
    pub kind: String,
    pub adapter_id: String,
    pub address: Option<String>,
    pub timestamp_micros: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Change {
    pub fn new(kind: &str, adapter_id: String, address: Option<String>) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        Self {
            kind: kind.into(),
            adapter_id,
            address,
            timestamp_micros: now.as_micros().min(i64::MAX as u128) as i64,
        }
    }

    pub fn row(&self) -> crate::Network {
        crate::Network {
            change: Some(self.kind.clone()),
            adapter_id: Some(self.adapter_id.clone()),
            address: self.address.clone(),
            ..crate::Network::default()
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::{harmos::Message, telemetry::TelemetryRow};
    #[test]
    fn compact_event() {
        let change = Change {
            kind: "address_removed".into(),
            adapter_id: "test".into(),
            address: Some("192.0.2.9".into()),
            timestamp_micros: 99,
        };
        let row = change.row();
        assert!(row == crate::Network::decode(&row.encode()).unwrap());
        assert_eq!(row.change.as_deref(), Some("address_removed"));
        assert_eq!(row.adapter_id.as_deref(), Some("test"));
        assert_eq!(row.address.as_deref(), Some("192.0.2.9"));
        assert!(row.internet.is_none());
        assert!(!row.into_row().contains_key("timestamp"));
    }
}
