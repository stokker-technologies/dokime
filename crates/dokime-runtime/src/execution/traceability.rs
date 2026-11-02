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
use std::sync::{Mutex, PoisonError};

// Third Party Imports

// Local Imports
use crate::execution::log::Level;
use crate::transport::ExecutionTransport;
use dokime_domain::model::traceability::{TraceabilityRow, TraceabilitySection, upsert_section, validate_section};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(crate) struct TraceabilityIndex {
    sections: Vec<TraceabilitySection>,
}

pub struct TraceabilityHandle<'a> {
    index: &'a Mutex<TraceabilityIndex>,
    transport: &'a dyn ExecutionTransport,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<'a> TraceabilityHandle<'a> {
    pub(crate) fn new(index: &'a Mutex<TraceabilityIndex>, transport: &'a dyn ExecutionTransport) -> Self {
        Self { index, transport }
    }

    /// Add Or Replace A Complete Section By Identifier.
    pub fn section(&mut self, section: TraceabilitySection) {
        if let Err(reason) = validate_section(&section) {
            self.refuse(reason);
            return;
        }
        let mut index = self.index.lock().unwrap_or_else(PoisonError::into_inner);
        upsert_section(&mut index.sections, section.clone());
        self.transport.traceability(vec![section]);
    }

    /// Add Or Replace A Row In An Existing Section By Identifier.
    pub fn row(&mut self, section_id: &str, row: TraceabilityRow) {
        let mut index = self.index.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(section) = index.sections.iter_mut().find(|section| section.identifier.as_str() == section_id) else {
            self.refuse(format!("section \"{section_id}\" is missing; add it before its rows"));
            return;
        };
        match section.rows.iter_mut().find(|existing| existing.identifier == row.identifier) {
            Some(existing) => *existing = row,
            None => section.rows.push(row),
        }
        self.transport.traceability(vec![section.clone()]);
    }

    fn refuse(&self, reason: String) {
        self.transport.log(Level::Error, format!("traceability was not changed: {reason}"));
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::recording::RecordingTransport;
    use dokime_domain::model::values::Value;

    #[test]
    fn edits_replace_identity_and_preserve_unrelated_rows_and_sections() {
        let index = Mutex::default();
        let transport = RecordingTransport::default();
        let mut handle = TraceabilityHandle::new(&index, &transport);
        handle.section(TraceabilitySection::new("dut", "Device Under Test"));
        handle.row("dut", TraceabilityRow::text("serial", "Serial", "SN-7"));
        handle.row("dut", TraceabilityRow::text("firmware", "Firmware", "v1"));
        handle.section(TraceabilitySection::new("bench", "Bench"));
        handle.row("dut", TraceabilityRow::text("serial", "Serial", "SN-8"));
        let captured = transport.traceability();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0].rows.len(), 2);
        assert_eq!(captured[0].rows[0].value, Value::from("SN-8"));
        assert_eq!(captured[0].rows[1].value, Value::from("v1"));
        handle.section(TraceabilitySection::new("dut", "Replacement Device"));
        let captured = transport.traceability();
        assert_eq!(captured.len(), 2);
        assert!(captured[0].rows.is_empty());
        assert_eq!(captured[0].label, "Replacement Device");
        assert_eq!(captured[1].identifier.as_str(), "bench");
    }

    #[test]
    fn refused_edits_preserve_captured_sections() {
        let index = Mutex::default();
        let transport = RecordingTransport::default();
        let mut handle = TraceabilityHandle::new(&index, &transport);
        handle.section(TraceabilitySection::new("dut", "Device Under Test"));
        handle.row("missing", TraceabilityRow::text("serial", "Serial", "SN-7"));
        handle.section(TraceabilitySection::new("host", "Host"));
        let mut duplicate = TraceabilitySection::new("dut", "Duplicate");
        duplicate.row(TraceabilityRow::text("serial", "Serial", "SN-7"));
        duplicate.row(TraceabilityRow::text("serial", "Serial", "SN-8"));
        handle.section(duplicate);
        let captured = transport.traceability();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].label, "Device Under Test");
        assert!(captured[0].rows.is_empty());
        assert_eq!(transport.diagnostics().len(), 3);
    }
}
