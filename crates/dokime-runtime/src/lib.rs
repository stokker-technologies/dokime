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

#[cfg(feature = "testing")]
#[doc(hidden)]
pub mod testing;

// Public Within The Unit-Test Build Only: Each Target Uses Its Own Part Of The Shared Story Support.
#[cfg(all(test, feature = "native"))]
#[path = "../tests/support/services/mod.rs"]
pub mod service_support;

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
pub mod host;
pub mod model;
pub mod parameters;
pub mod paths;
pub mod plugin;

#[cfg(feature = "native")]
pub mod prelude;

#[cfg(feature = "native")]
pub mod process;
pub mod prompts;
pub mod publication;
pub mod records;

pub mod analysis;
#[cfg(feature = "storage")]
pub mod storage;
pub mod telemetry;
pub mod traceability;
pub mod transport;
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

#[cfg(feature = "native")]
pub mod app;

#[cfg(feature = "configuration")]
pub mod config;

#[cfg(feature = "native")]
mod persistence;

#[cfg(feature = "native")]
mod plot;

#[cfg(feature = "native")]
pub mod plugins;

#[cfg(feature = "native")]
pub mod report;

#[cfg(feature = "native")]
pub mod runtime;

#[cfg(feature = "native")]
mod services;

#[cfg(feature = "native")]
pub mod state;

#[cfg(feature = "native")]
pub mod streams;

#[cfg(feature = "native")]
pub mod transactions;

#[cfg(feature = "native")]
pub mod brand {
    pub use crate::domains::installed_plugins::brand::theme;
}

/* === Re-Exports === */

pub use definition::{Parameter, ParameterDefault, ParameterExpansion};
pub use dokime_domain::DEV_PROTOCOL;
pub use dokime_domain::Error as ApiError;
pub use dokime_domain::READINESS_PROTOCOL;
pub use dokime_domain::model::appearance::{HostCardBlock, HostCardBody, HostCardKeyValue, HostCardMetric, HostCardTone};
pub use dokime_domain::model::diagnostics::*;
pub use dokime_domain::model::execution::*;
pub use dokime_domain::model::expressions;
pub use dokime_domain::model::identity::*;
pub use dokime_domain::model::secrets::*;

pub use dokime_domain::model::settings;
pub use dokime_domain::model::time::*;
pub use dokime_domain::model::validation::*;
pub use dokime_domain::model::values;
pub use dokime_domain::model::values::*;
pub use dokime_operations as operations;

// Several model globs carry a `wire` module; the value wire form is the one this crate names.
pub use dokime_domain::model::values::wire;
pub use error::{Error, Result};
pub use execution::Slot;
pub use parameters::{FromParameters, ParameterChoice, ParameterHandle, ParameterValue};
pub use publication::Publishable;

// The Harmos plugin model, with its own type identity: a Dokime plugin is a
// Harmos plugin, its device operations are Harmos operations, and its domains
// reach the host as Harmos contributions.
pub use contributions::{Evidence, Records, RunStart, Settings, Suites};
pub use harmos::Plugin;
pub use harmos::catalog::RequirementEntry as Requirement;
pub use harmos::operations::{CallContext, Operation, OperationDeclaration};
pub use harmos::plugin::{Contribution, Grant};

pub use harmos;
#[cfg(feature = "native")]
pub use harmos::sidecar::{Refusal, Supervisor};
pub use values::{FieldError, FieldValue};

// The authoring attributes, re-exported so suites and sidecars write
// `#[dokime_runtime::…]` and never name the plumbing crate. They live in the macro
// namespace and add no name to the type census, and they are behind the
// `derive` feature the way serde ships its derives: the traits under
// [`plugin`] are the API, and these are the path most authors take to them.
#[cfg(feature = "derive")]
pub use dokime_macros::{
    catalog, choice, criterion, listener, message, observer, operation, parameters, plugin, prompt, record, row, section, settings, step,
    stream, suite, template,
};
pub use harmos::{Message, StreamSpec, schema};

// Generated authoring code resolves implementation dependencies through the SDK.
pub use dokime_domain as __domain;
pub use harmos as __harmos;
pub use prost as __prost;

#[cfg(feature = "native")]
pub use tokio as __tokio;

#[cfg(feature = "native")]
pub(crate) use domains::{
    documents, evidence as evidence_runtime, execution as execution_runtime, exports, installed_plugins, parameters as parameter_runtime,
    prompts as prompt_runtime, secrets, telemetry as telemetry_runtime, traceability as traceability_runtime,
};

#[cfg(feature = "native")]
pub(crate) use runtime::command as commands;

#[cfg(feature = "native")]
pub(crate) use runtime::command::CommandReceipt;

#[cfg(feature = "native")]
pub(crate) use runtime::query as queries;

#[cfg(feature = "native")]
pub(crate) use runtime::{handle, state as runtime_state};

#[cfg(feature = "native")]
pub use app::Dokime;

#[cfg(feature = "native")]
pub use config::{InstalledSidecar, RuntimeConfig, SidecarNoticeHandler};

#[cfg(feature = "native")]
pub use dokime_domain::model::plugins::InstalledPlugin;

#[cfg(feature = "native")]
pub use domains::documents::{CurrentDocumentCapture, ExecutionFileFollower, ExecutionFileStream, PreparedCurrentDocument};

#[cfg(feature = "native")]
pub use domains::execution::outputs::{ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry, ExecutionFileStreamState};

#[cfg(feature = "native")]
pub use domains::execution::provider::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};

#[cfg(feature = "native")]
pub use domains::telemetry::LeaseFact;

#[cfg(feature = "native")]
pub(crate) use runtime::RuntimeState;

#[cfg(feature = "native")]
pub(crate) use runtime::command::{internal_rejection, rejected_receipt, unavailable_rejection};

#[cfg(feature = "native")]
pub use runtime::{OpenedReport, ReportOrigin, Runtime, RuntimeHandle};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
// Generated authoring paths resolve the same way inside and outside this crate.
extern crate self as dokime_runtime;

// Local Imports

/* === Constants === */

pub const MACRO_CONTRACT: u32 = 28;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
