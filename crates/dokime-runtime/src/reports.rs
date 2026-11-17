/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Publication

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

// Third Party Imports
use harmos_protocol::model::FileReference;

// Local Imports
use crate::{Error, Result, host::lock};

use dokime_storage::{Report, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct ReportManager {
    store: Store,
    state: Arc<Mutex<Publication>>,
}
#[derive(Default)]
struct Publication {
    closed: bool,
    temporary: BTreeMap<String, FileReference>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ReportManager {
    pub(crate) fn new(store: Store) -> Self {
        Self {
            store,
            state: Arc::new(Mutex::new(Publication::default())),
        }
    }
    pub(crate) fn publish_prepared(&self, prepared: Report) -> Result<Report> {
        let state = lock(&self.state)?;
        if state.closed {
            return Err(Error::Invalid("Publish Report: Reconnect to a Running Host".into()));
        }
        Ok(self.store.publish_report(
            &prepared.execution,
            &prepared.execution_seal,
            prepared.document.ok_or_else(|| Error::Invalid("Publish Report: Restore Its Frozen Document".into()))?,
            prepared.html,
        )?)
    }
    pub(crate) fn retain(&self, reference: FileReference, limit: usize) -> Result<()> {
        let identifier =
            reference.metadata.as_ref().ok_or_else(|| Error::Invalid("Retain Report: Supply Its Temporary Identity".into()))?.id.clone();
        let mut state = lock(&self.state)?;
        if state.closed {
            return Err(Error::Invalid("Retain Report: Reconnect to a Running Host".into()));
        }
        if limit == 0 {
            return Err(Error::Invalid("Retain Report: Configure a Positive Temporary Report Limit".into()));
        }
        if state.temporary.len() >= limit
            && !state.temporary.contains_key(&identifier)
            && let Some(first) = state.temporary.keys().next().cloned()
        {
            state.temporary.remove(&first);
        }
        state.temporary.insert(identifier, reference);
        Ok(())
    }
    pub(crate) fn temporary(&self, identifier: &str) -> Result<Option<FileReference>> {
        let state = lock(&self.state)?;
        if state.closed {
            return Err(Error::Invalid("Read Temporary Report: Reconnect to a Running Host".into()));
        }
        Ok(state.temporary.get(identifier).cloned())
    }
    pub(crate) fn shutdown(&self) -> Result<()> {
        let mut state = lock(&self.state)?;
        state.closed = true;
        state.temporary.clear();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
