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
use crate::{Host, Result, dispatch::Dispatch};
use dokime_protocol::v1;
use tokio_util::sync::CancellationToken;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub async fn analyze(&self, plugin: &str, operation: &str, request: v1::AnalysisRequest) -> Result<v1::AnalysisResponse> {
        let dispatch = Dispatch {
            entry: self.entry(plugin)?,
            resources: None,
            cancellation: CancellationToken::new(),
            settings: self.inner.settings.placement.clone(),
            clock: self.inner.settings.clock.clone(),
        };
        dispatch.invoke(operation, request).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
