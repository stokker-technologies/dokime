/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Local Imports
use crate::{Host, Error, Result, host::lock};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub async fn detach_plugin(&self, identifier: &str) -> Result<()> {
        if lock(&self.inner.active)?.values().any(|active| active.plugin == identifier) { return Err(Error::Invalid("Cancel the Plugin's Active Executions and Wait for Cleanup Before Changing Its Entry".into())); }
        let entry = lock(&self.inner.plugins)?.get(identifier).cloned();
        if let Some(entry) = entry {
            if let Some(mut process) = entry.process.lock().await.take() { process.close().await?; }
            lock(&self.inner.plugins)?.remove(identifier);
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
