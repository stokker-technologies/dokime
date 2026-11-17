/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;
use std::{
    io::Write,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use prost::Message;
use tokio::io::AsyncReadExt;
use tracing::instrument::WithSubscriber;

// Local Imports
use dokime_runtime::{
    ControlledClock, HostSettings, PlacementSettings, SchedulerClock,
    services::{self, ClientSettings, WallClock},
};

use dokime_storage::StoredExecutionFile;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

// Admissions and flush commits briefly hold the store's blocking locks; worker threads keep
// plugin transport, clients and the flush timer running meanwhile, and connections decode in parallel.
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let mut endpoint = None;
    let mut root = None;
    let mut controlled = false;
    let mut remove_credentials = false;
    let mut execution_files = Vec::new();
    let mut capture = dokime_runtime::CaptureSettings::default();
    let mut environment = None;
    let mut credentials = dokime_runtime::CredentialNamespace::default();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--endpoint" => endpoint = Some(arguments.next().ok_or("Start Host: Supply --endpoint HOST:PORT")?.parse()?),
            "--storage" => root = Some(PathBuf::from(arguments.next().ok_or("Start Host: Supply --storage PATH")?)),
            "--controlled-clock" => controlled = true,
            "--capture-root" => capture.roots.push(PathBuf::from(arguments.next().ok_or("Start Host: Supply --capture-root PATH")?)),
            "--remove-credentials" => remove_credentials = true,
            "--attach-execution-file" => {
                execution_files.push(PathBuf::from(arguments.next().ok_or("Register Execution File: Supply Its Encoded Metadata Path")?))
            }
            "--credential-namespace" => {
                credentials =
                    dokime_runtime::CredentialNamespace::new(arguments.next().ok_or("Start Host: Supply --credential-namespace NAME")?)?
            }
            "--env-file" => environment = Some(PathBuf::from(arguments.next().ok_or("Start Host: Supply --env-file PATH")?)),
            _ => return Err(format!("Start Host: Unknown Argument {argument}").into()),
        }
    }
    let root = root.ok_or("Start Host: Supply --storage PATH")?;
    // Resolve Every Environment Read Before the Host Starts; Process Variables Take Precedence.
    let file = environment.as_deref().map(log_directives).transpose()?.flatten();
    let directives = std::env::var("RUST_LOG").ok().or(file).unwrap_or_default();
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .parse(directives)?;
    if remove_credentials {
        services::remove_credentials(&root, &credentials)?;
        return Ok(());
    }
    let clock: Arc<dyn SchedulerClock> = if controlled { Arc::new(ControlledClock::new(0)) } else { Arc::new(WallClock) };
    let host_settings = HostSettings {
        credentials,
        storage: dokime_storage::storage_defaults(),
        clock,
        placement: PlacementSettings {
            reconnect_delay: Duration::from_millis(25),
            stream_window: NonZeroUsize::new(8).unwrap(),
            interval_window: dokime_runtime::INTERVAL_WINDOW,
            interval_lag: dokime_runtime::INTERVAL_LAG,
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        capture,
    };
    let settings = ClientSettings {
        read_timeout: Duration::from_secs(30),
        mutation_timeout: Duration::from_secs(30),
        watch_timeout: Duration::from_secs(3600),
        max_cases: 1000,
        page_items: 1000,
        message_bytes: 4 * 1024 * 1024,
        stream_buffer: 8,
    };
    if !execution_files.is_empty() {
        let host = dokime_runtime::Host::open(&root, host_settings, settings).await?;
        for path in execution_files {
            let bytes = std::fs::read(path)?;
            let stored = StoredExecutionFile::decode(bytes.as_slice())?;
            let file = stored.file.ok_or("Register Execution File: Supply Its Native File Reference")?;
            let execution = file
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.scope.as_deref())
                .ok_or("Register Execution File: Select Its Execution Scope")?
                .to_owned();
            let metadata = stored.metadata.ok_or("Register Execution File: Supply Its Logical Path Metadata")?;
            host.attach_execution_file(&execution, metadata, file).await?;
        }
        host.shutdown().await?;
        return Ok(());
    }
    let endpoint = endpoint.ok_or("Start Host: Supply --endpoint HOST:PORT")?;
    use tracing_subscriber::prelude::*;
    // Startup Failures Reach Standard Error With Their Cause Before the Status Layer Exists.
    let startup = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr).with_filter(tracing::level_filters::LevelFilter::ERROR));
    let server = services::serve(endpoint, &root, host_settings, settings).with_subscriber(startup).await?;
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(filter).with(server.status().layer()))?;
    tracing::info!("Execution Host Is Ready");
    println!("http://{}", server.endpoint());
    std::io::stdout().flush()?;
    let mut input = [0u8; 1];
    while tokio::io::stdin().read(&mut input).await? != 0 {}
    server.close().await?;
    Ok(())
}

// The Host Reads Only RUST_LOG; Any Other Variable in the Explicit File Is Refused.
fn log_directives(path: &Path) -> Result<Option<String>, Box<dyn Error>> {
    if std::fs::metadata(path)?.len() > 1024 * 1024 {
        return Err(format!("Read Environment File {}: Select a File of at Most 1 MiB", path.display()).into());
    }
    let mut directives = None;
    for (index, line) in std::fs::read_to_string(path)?.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let refused = |message: &str| format!("Read Environment File {}: {message} on Line {}", path.display(), index + 1);
        let (name, value) = line.split_once('=').ok_or_else(|| refused("Write NAME=VALUE"))?;
        if name.trim() != "RUST_LOG" {
            return Err(refused(&format!("Remove Unknown Variable {}", name.trim())).into());
        }
        let value = value.trim();
        let value =
            ['"', '\''].iter().find_map(|quote| value.strip_prefix(*quote).and_then(|value| value.strip_suffix(*quote))).unwrap_or(value);
        if directives.replace(value.to_owned()).is_some() {
            return Err(refused("Remove the Repeated Variable RUST_LOG").into());
        }
    }
    Ok(directives)
}

/* === Tests === */
