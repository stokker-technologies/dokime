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
use std::sync::Weak;

// Third Party Imports

// Local Imports
use crate::{Error, Host, Result};
use dokime_protocol::plugin::{AnalysesComputeRequest, AnalysesComputeResponse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub async fn analyze(&self, plugin: &str, operation: &str, request: AnalysesComputeRequest) -> Result<AnalysesComputeResponse> {
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Analyze: Assemble the Host Context".into()))?;
        Ok(context.host.analyze(plugin, operation, request).await?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
