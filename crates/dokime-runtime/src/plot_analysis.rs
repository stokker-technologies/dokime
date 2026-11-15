/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Unary Plot Analysis Invocation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Host, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub async fn analyze(
        &self, plugin: &str, operation: &str, request: dokime_protocol::plugin::AnalysesComputeRequest,
    ) -> Result<dokime_protocol::plugin::AnalysesComputeResponse> {
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(std::sync::Weak::upgrade)
            .ok_or_else(|| crate::Error::Invalid("Analyze: Assemble the Host Context".into()))?;
        Ok(context.host.analyze(plugin, operation, request).await?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
