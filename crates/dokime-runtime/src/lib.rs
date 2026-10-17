/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![recursion_limit = "256"]

/* === Modules === */
#[macro_use]
mod macros;
pub mod artifacts;

pub mod catalog;

pub mod contributions;

pub mod definition;

pub mod diagnostics;

pub mod document;

pub mod domains;

pub mod error;

pub mod evidence;

pub mod execution;

#[cfg(feature = "native")]
pub mod files;

pub mod paths;

pub mod host;

pub mod parameters;

pub mod plugin;

#[cfg(feature = "native")]
pub mod prelude;

#[cfg(feature = "native")]
pub mod process;

pub mod prompts;

pub mod publication;

pub mod records;

#[cfg(feature = "storage")]
pub mod storage;

pub mod telemetry;

pub mod traceability;

pub mod transport;

pub mod analysis;

pub use dokime_contract::values;

pub mod __answer {
    pub trait Answer {
        type Ok;
        type Err;
    }

    impl<T, E> Answer for Result<T, E> {
        type Ok = T;
        type Err = E;
    }

    pub trait Initialized<T> {
        type Own;
        fn into_parts(self) -> (T, Self::Own);
    }

    impl<T> Initialized<T> for T {
        type Own = ();
        fn into_parts(self) -> (T, ()) {
            (self, ())
        }
    }

    impl<T, Own> Initialized<T> for (T, Own) {
        type Own = Own;
        fn into_parts(self) -> (T, Own) {
            self
        }
    }
}


#[cfg(feature = "host")]
pub mod app;
#[cfg(feature = "host")]
pub mod config;
pub use dokime_contract::operations;
#[cfg(feature = "host")]
mod persistence;
#[cfg(feature = "host")]
mod plot;
#[cfg(feature = "host")]
pub mod plugins;
#[cfg(feature = "host")]
pub mod report;
#[cfg(feature = "host")]
pub mod runtime;
#[cfg(feature = "host")]
mod services;
#[cfg(feature = "host")]
pub mod state;
#[cfg(feature = "host")]
pub mod streams;
#[cfg(feature = "host")]
pub mod transactions;

#[cfg(feature = "host")]
pub mod brand {
    pub use crate::domains::installed_plugins::brand::theme;
}

/* === Re-Exports === */
pub use dokime_contract::core;

pub use dokime_contract::expressions;


pub use definition::{Parameter, ParameterDefault, ParameterExpansion};
pub use dokime_contract::Error as ApiError;
pub use dokime_contract::core::*;
pub use dokime_contract::settings;
pub use dokime_contract::{DEV_PROTOCOL, READINESS_PROTOCOL};
pub use error::{Error, Result};
pub use execution::Slot;
pub use parameters::{FromParameters, ParameterChoice, ParameterHandle, ParameterValue};
pub use publication::Publishable;

// The Harmos plugin model, with its own type identity: a Dokime plugin is a
// Harmos plugin, its device operations are Harmos operations, and its domains
// reach the host as Harmos contributions.
pub use contributions::{Evidence, Records, Settings, Suites};
pub use harmos::Plugin;
pub use harmos::catalog::RequirementEntry as Requirement;
pub use harmos::operations::{CallContext, Declares, Operation};
pub use harmos::plugin::{Contribution, Grant};
#[cfg(feature = "native")]
pub use harmos::sidecar::{Refusal, Supervisor};

pub use harmos;
pub use values::{FieldError, FieldValue};

// The authoring attributes, re-exported so suites and sidecars write
// `#[dokime_runtime::…]` and never name the plumbing crate. They live in the macro
// namespace and add no name to the type census, and they are behind the
// `derive` feature the way serde ships its derives: the traits under
// [`plugin`] are the API, and these are the path most authors take to them.
#[cfg(feature = "derive")]
pub use dokime_macros::{
    catalog, choice, criterion, listener, message, observer, operation, parameters, plugin, prompt,
    record, row, section, settings, step, stream, suite, template,
};
pub use harmos::{Message, StreamSpec, schema};

// Generated authoring code resolves implementation dependencies through the SDK.

pub use harmos as __harmos;

pub use prost as __prost;

#[cfg(feature = "native")]
pub use tokio as __tokio;

#[cfg(feature = "host")]
pub(crate) use domains::{
    credentials, documents, evidence as evidence_runtime, execution as execution_runtime, exports, installed_plugins,
    parameters as parameter_runtime, prompts as prompt_runtime, telemetry as telemetry_runtime, traceability as traceability_runtime,
};
#[cfg(feature = "host")]
pub(crate) use runtime::command as commands;
#[cfg(feature = "host")]
pub(crate) use runtime::command::{CommandContext, CommandId, CommandReceipt, JobRunId};
#[cfg(feature = "host")]
pub(crate) use runtime::query as queries;
#[cfg(feature = "host")]
pub(crate) use runtime::{handle, state as runtime_state};


#[cfg(feature = "host")]
pub use app::Dokime;
#[cfg(feature = "host")]
pub use config::{InstalledSidecar, RuntimeConfig, RuntimeCredentialConfig, SidecarNoticeHandler};

#[cfg(feature = "host")]
pub use dokime_contract::domains::InstalledPlugin;
#[cfg(feature = "host")]
pub use domains::documents::{
    CurrentDocumentCapture, CurrentDocumentTelemetryCut, ExecutionFileFollower,
    ExecutionFileStream, PreparedCurrentDocument,
};
#[cfg(feature = "host")]
pub use domains::execution::outputs::{
    ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry, ExecutionFileStreamState,
};
#[cfg(feature = "host")]
pub use domains::execution::provider::{
    ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture,
};
#[cfg(feature = "host")]
pub use domains::telemetry::LeaseFact;

#[cfg(feature = "host")]
pub(crate) use runtime::RuntimeState;
#[cfg(feature = "host")]
pub(crate) use runtime::command::{internal_rejection, rejected_receipt, unavailable_rejection};
#[cfg(feature = "host")]
pub use runtime::{OpenedReport, ReportOrigin, Runtime, RuntimeHandle};


/* === Imports === */
// Standard Library Imports

// Third Party Imports
// Generated authoring paths resolve the same way inside and outside this crate.
extern crate self as dokime_runtime;
// Local Imports

/* === Constants === */
pub const MACRO_CONTRACT: u32 = 26;

/* === Macros === */
/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
