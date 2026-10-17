/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod appearance;
pub mod backend;
pub mod catalog;
pub mod checkpoint;
pub mod common;
pub mod content;
pub mod cursor;
pub mod documents;
pub mod evidence;
pub mod execution;
pub mod files;
pub mod history;
pub mod media;
pub mod notes;
pub mod observation;
pub mod planning;
pub mod plots;
pub mod plugins;
pub mod process;
pub mod prompts;
pub mod query;
pub mod reports;
pub mod runtime;
pub mod schema;
pub mod settings;
pub mod telemetry;
pub mod value;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use harmos::catalog::{Member, Visitor};
use harmos::operations::{
    OperationPolicy, RestartPolicy, RetryPolicy, WireDeclaration, WireOperation,
};
use harmos::schema::Described;
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */
pub const PACKAGE: &str = "dokime.v1";

// Bounds One Ordinary Request From Admission To Its Response.
pub const ORDINARY: OperationPolicy = policy(Duration::from_secs(10));

// Bounds A Transfer, Including Every Streamed Item.
pub const TRANSFER: OperationPolicy = policy(Duration::from_secs(300));

// Bounds A Plugin Installation, Which Ends When It Settles Or The Caller Cancels:
// A Slow Machine May Take Minutes To Stage And Activate A Debug Plugin, So The
// Hour Is A Safety Net Rather Than A Practical Limit.
pub const INSTALLATION: OperationPolicy = policy(Duration::from_secs(3_600));

// Bounds A Follow Stream, Which Ends When The Caller Cancels Or The Runtime Stops.
pub const FOLLOW: OperationPolicy = policy(Duration::from_secs(86_400));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Endpoints;

pub struct Declaration {
    pub path: &'static str,
    pub access: Access,
    pub local_only: bool,
    pub wire: WireDeclaration,
    pub rust_type: fn() -> &'static str,
}

/* === Enums === */
#[cfg(feature = "descriptors")]
#[derive(Debug, thiserror::Error)]
pub enum DescriptorError {
    #[error(transparent)]
    Contract(#[from] harmos::operations::ContractError),
    #[error(transparent)]
    Export(#[from] harmos::grpc::ExportError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Read,
    Control,
    Admin,
}

#[harmos::catalog]
pub enum Operations {
    Host(harmos::operations::host::Operations),
    Health(runtime::GetHealth),
    Identity(runtime::GetIdentity),
    #[nested]
    Appearance(appearance::Operations),
    #[nested]
    Backend(backend::Operations),
    #[nested]
    Catalog(catalog::Operations),
    #[nested]
    Content(content::Operations),
    #[nested]
    Documents(documents::Operations),
    #[nested]
    Evidence(evidence::Operations),
    #[nested]
    Execution(execution::Operations),
    #[nested]
    Files(files::Operations),
    #[nested]
    History(history::Operations),
    #[nested]
    Media(media::Operations),
    #[nested]
    Notes(notes::Operations),
    #[nested]
    Observation(observation::Operations),
    #[nested]
    Planning(planning::Operations),
    #[nested]
    Plots(plots::Operations),
    #[nested]
    Plugins(plugins::Operations),
    #[nested]
    Prompts(prompts::Operations),
    #[nested]
    Query(query::Operations),
    #[nested]
    Reports(reports::Operations),
    #[nested]
    Settings(settings::Operations),
    #[nested]
    Telemetry(telemetry::Operations),
}

/* === Traits === */
// The Dokime authorization of one public operation, enforced at the listener.
pub trait Endpoint: WireOperation {
    const PATH: &'static str;
    const ACCESS: Access;
    // Served only to a loopback listener, because the reply names a host path.
    const LOCAL_ONLY: bool = false;
}

/* === Implementations === */
impl Endpoint for harmos::operations::host::Operations {
    const PATH: &'static str = "/harmos.host.v1.Host/Operations";
    const ACCESS: Access = Access::Read;
}

impl Visitor for Endpoints {
    type Entry = Declaration;
}

impl<O> Member<O> for Endpoints
where
    O: Endpoint,
    O::Request: Described,
    O::Response: Described,
{
    const ENTRY: Declaration = Declaration {
        path: O::PATH,
        access: O::ACCESS,
        local_only: O::LOCAL_ONLY,
        wire: WireDeclaration::of::<O>(),
        rust_type: std::any::type_name::<O>,
    };
}

/* === Public Functions === */
// The descriptor set and readable `.proto` sources of every public operation.
#[cfg(feature = "descriptors")]
pub fn artifacts() -> Result<harmos::grpc::Artifacts, DescriptorError> {
    let contracts = harmos::operations::Contracts::of::<Operations>()?;
    Ok(harmos::grpc::Artifacts::new(&contracts)?)
}

/* === Private Functions === */
const fn policy(timeout: Duration) -> OperationPolicy {
    OperationPolicy {
        timeout,
        attempt_timeout: timeout,
        retries: RetryPolicy::Never,
        restart: RestartPolicy::Never,
    }
}

/* === Tests === */
