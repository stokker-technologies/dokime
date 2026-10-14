/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation Initializers Capture Runtime Handles; Proving Their Futures Send Needs This Depth.
#![recursion_limit = "256"]

/* === Modules === */
#[macro_use]
mod macros;
pub mod app;
pub mod config;
pub mod domains;

pub mod definition;

pub mod document;
mod error;

pub mod execution;
mod operations;
mod persistence;
mod plot;
pub mod plugins;

pub mod records;
pub mod report;
pub mod runtime;
mod services;
pub mod state;
pub mod streams;
pub mod traceability;
pub mod transactions;

pub mod brand {
    pub use crate::domains::installed_plugins::brand::theme;
}

/* === Re-Exports === */
// Crate-private vocabulary keeps implementation imports concise while the
// public module tree exposes only the anchors declared above.
pub(crate) use domains::{
    credentials, documents, evidence, execution as execution_runtime, exports, installed_plugins,
    parameters, prompts, telemetry as telemetry_runtime, traceability as traceability_runtime,
};
pub(crate) use runtime::command as commands;
pub(crate) use runtime::command::{CommandContext, CommandId, CommandReceipt, JobRunId};
pub(crate) use runtime::query as queries;
pub(crate) use runtime::{handle, state as runtime_state};

pub use app::Dokime;
pub use config::{InstalledSidecar, RuntimeConfig, RuntimeCredentialConfig, SidecarNoticeHandler};
pub use dokime::{
    ApiError, StreamSpec, analysis, artifacts, core, expressions, plugin, settings, telemetry,
    transport,
};
pub use dokime_contract::domains::InstalledPlugin;
pub use domains::documents::{
    CurrentDocumentCapture, CurrentDocumentTelemetryCut, ExecutionFileFollower,
    ExecutionFileStream, PreparedCurrentDocument,
};
pub use domains::execution::outputs::{
    ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry, ExecutionFileStreamState,
};
pub use domains::execution::provider::{
    ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture,
};
pub use domains::telemetry::LeaseFact;
pub use error::{Error, Result};
pub(crate) use runtime::RuntimeState;
pub(crate) use runtime::command::{internal_rejection, rejected_receipt, unavailable_rejection};
pub use runtime::{Runtime, RuntimeHandle};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */
// Display and Error for the closed refusal enums the catalogs return. It
// travelled here with the fold: the transactions that use it now live beside
// the runtime that applies them.
/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
