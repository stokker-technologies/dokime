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
pub mod effects;
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
use harmos::operations::OperationDeclaration;
use harmos::operations::OperationPolicy;
use harmos::operations::RetryPolicy;

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

// Bounds How Long A Stopped Run Has To Answer Before The Host Settles It As Stopped. It Fits
// Inside An `ORDINARY` Stop With Room To Save The Evidence; The Desktop And Server Stops Build On It.
pub const STOP_SETTLEMENT: OperationPolicy = policy(Duration::from_secs(5));

// Operations A Remote Listener Never Serves: They Reach The Host's Filesystem, Screen,
// Camera, Microphone Or Listener Settings. A Remote Call's Loopback Address Grants No Native Trust.
pub const LOCAL_ONLY: &[&str] = &[
    <files::FileWatch as OperationDeclaration>::ID,
    <media::Screenshot as OperationDeclaration>::ID,
    <media::Photo as OperationDeclaration>::ID,
    <media::AudioRecording as OperationDeclaration>::ID,
    <media::ScreenRecording as OperationDeclaration>::ID,
    <media::CameraRecording as OperationDeclaration>::ID,
    <remote::ReadListener as OperationDeclaration>::ID,
    <remote::UpdateListener as OperationDeclaration>::ID,
    <remote::CreatePairing as OperationDeclaration>::ID,
    <remote::ListPairedClients as OperationDeclaration>::ID,
    <remote::RevokePairedClient as OperationDeclaration>::ID,
    <remote::ListSessions as OperationDeclaration>::ID,
    <remote::RegenerateCertificate as OperationDeclaration>::ID,
    <remote::RenamePairedClient as OperationDeclaration>::ID,
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/* === Implementations === */

/* === Public Functions === */

/// The gRPC method path an operation declares, such as `/dokime.v1.Notes/CreateNote`.
pub fn path<O: OperationDeclaration>() -> String {
    O::GRPC.expect("product operations declare their gRPC binding").path()
}

pub fn local_only(id: &str) -> bool {
    LOCAL_ONLY.contains(&id)
}

/* === Private Functions === */

const fn policy(timeout: Duration) -> OperationPolicy {
    OperationPolicy {
        timeout,
        attempt_timeout: timeout,
        cancellation_timeout: std::time::Duration::ZERO,
        retries: RetryPolicy::Never,
    }
}

/* === Tests === */
