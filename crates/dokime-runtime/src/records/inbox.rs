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
use std::sync::Arc;
// Third Party Imports

// Local Imports
use crate::records::Record;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct Deliveries {
    source: Arc<dyn RecordSource>,
}

/* === Enums === */

/* === Traits === */
pub trait RecordSource: Send + Sync {
    fn delivered(&self, index: usize) -> Option<Record>;

    fn next_record(&self, cursor: &mut usize) -> Option<Record> {
        let record = self.delivered(*cursor)?;
        *cursor += 1;
        Some(record)
    }
}

/* === Implementations === */
impl Deliveries {
    pub fn new(source: Arc<dyn RecordSource>) -> Self {
        Self { source }
    }

    pub fn source(&self) -> Arc<dyn RecordSource> {
        Arc::clone(&self.source)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
