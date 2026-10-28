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
pub mod content;
pub mod documents;
pub mod evidence;
pub mod execution;
pub mod files;
pub mod history;
#[cfg(feature = "runtime")]
pub mod hosting;
pub mod media;
pub mod notes;
pub mod observation;
pub mod planning;
pub mod plots;
pub mod plugins;
pub mod process;
pub mod prompts;
pub mod query;
pub mod records;
pub mod remote;
pub mod reports;
pub mod runtime;
pub mod secrets;
pub mod settings;
pub mod telemetry;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use harmos_domain::catalog::Member;
use harmos_domain::catalog::Visitor;
use harmos_domain::operations::OperationDeclaration;
use harmos_domain::operations::OperationPolicy;
use harmos_domain::operations::RetryPolicy;
use harmos_domain::operations::WireDeclaration;
use harmos_domain::schema::Described;

// Local Imports

/* === Constants === */

pub const PACKAGE: &str = "dokime.v1";

// Bounds One Ordinary Request From Admission To Its Response.
pub const ORDINARY: OperationPolicy = policy(Duration::from_secs(10));

// Bounds A Mutation That Writes Many Rows Or Files, Such As Creating, Copying Or
// Deleting Whole Cycles And Executions, Materializing Cases Or Moving Storage.
pub const MUTATION: OperationPolicy = policy(Duration::from_secs(120));

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
    pub local_only: bool,
    pub wire: WireDeclaration,
    pub rust_type: fn() -> &'static str,
}

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    Host(harmos::builtins::host::Catalog),
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
    Remote(remote::Operations),

    #[nested]
    Secrets(secrets::Operations),

    #[nested]
    Settings(settings::Operations),

    #[nested]
    Telemetry(telemetry::Operations),
}

/* === Traits === */

// Public Transport Binding And Its Native-Only Boundary.
pub trait Endpoint: OperationDeclaration {
    const PATH: &'static str;
    // Native Listener Trust Never Comes From A Remote Call's Loopback Address.
    const LOCAL_ONLY: bool = false;
}

/* === Implementations === */

impl Endpoint for harmos::builtins::host::Catalog {
    const PATH: &'static str = "/harmos.host.v1.Host/Catalog";
}

impl Visitor for Endpoints {
    type Entry = Declaration;
}

impl<O> Member<O> for Endpoints
where
    O: Endpoint,
    O::Response: Described,
{
    const ENTRY: Declaration = Declaration {
        path: O::PATH,
        local_only: O::LOCAL_ONLY,
        wire: WireDeclaration::of::<O>(),
        rust_type: std::any::type_name::<O>,
    };
}

/* === Public Functions === */

/* === Private Functions === */

const fn policy(timeout: Duration) -> OperationPolicy {
    OperationPolicy {
        timeout,
        attempt_timeout: timeout,
        retries: RetryPolicy::Never,
    }
}

/* === Tests === */
