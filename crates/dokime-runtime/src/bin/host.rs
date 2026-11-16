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
use std::error::Error as StdError;
use std::{
    io::Write,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use tokio::io::AsyncReadExt;

// Local Imports
use dokime_runtime::{
    ControlledClock, HostSettings, PlacementSettings, SchedulerClock,
    services::{self, ClientSettings, WallClock},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn StdError>> {
    let mut arguments = std::env::args().skip(1);
    let mut endpoint = None;
    let mut root = None;
    let mut controlled = false;
    let mut remove_credentials = false;
    let mut capture = dokime_runtime::CaptureSettings::default();
    let mut environment = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--endpoint" => endpoint = Some(arguments.next().ok_or("Start Host: Supply --endpoint HOST:PORT")?.parse()?),
            "--storage" => root = Some(PathBuf::from(arguments.next().ok_or("Start Host: Supply --storage PATH")?)),
            "--controlled-clock" => controlled = true,
            "--capture-root" => capture.roots.push(PathBuf::from(arguments.next().ok_or("Start Host: Supply --capture-root PATH")?)),
            "--remove-credentials" => remove_credentials = true,
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
        services::remove_credentials(&root)?;
        return Ok(());
    }
    let endpoint = endpoint.ok_or("Start Host: Supply --endpoint HOST:PORT")?;
    let clock: Arc<dyn SchedulerClock> = if controlled { Arc::new(ControlledClock::new(0)) } else { Arc::new(WallClock) };
    let host_settings = HostSettings {
        storage: dokime_storage::storage_defaults(),
        clock,
        placement: PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_millis(25),
            stream_window: NonZeroUsize::new(8).unwrap(),
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
    use tracing_subscriber::prelude::*;
    let server = services::serve(endpoint, &root, host_settings, settings).await?;
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
fn log_directives(path: &Path) -> Result<Option<String>, Box<dyn StdError>> {
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
