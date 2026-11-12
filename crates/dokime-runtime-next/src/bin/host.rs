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
use std::{io::Write, num::NonZeroUsize, path::PathBuf, sync::Arc, time::Duration};

// Third Party Imports
use tokio::io::AsyncReadExt;

// Local Imports
use dokime_runtime_next::{
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let mut endpoint = None;
    let mut root = None;
    let mut controlled = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--endpoint" => endpoint = Some(arguments.next().ok_or("Start Host: Supply --endpoint HOST:PORT")?.parse()?),
            "--storage" => root = Some(PathBuf::from(arguments.next().ok_or("Start Host: Supply --storage PATH")?)),
            "--controlled-clock" => controlled = true,
            _ => return Err(format!("Start Host: Unknown Argument {argument}").into()),
        }
    }
    let endpoint = endpoint.ok_or("Start Host: Supply --endpoint HOST:PORT")?;
    let root = root.ok_or("Start Host: Supply --storage PATH")?;
    let clock: Arc<dyn SchedulerClock> = if controlled { Arc::new(ControlledClock::new(0)) } else { Arc::new(WallClock) };
    let host_settings = HostSettings {
        clock,
        placement: PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_millis(25),
            stop_grace: Duration::from_secs(30),
            stream_window: NonZeroUsize::new(8).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
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
    let diagnostics = services::Diagnostics::new(host_settings.clock.clone(), NonZeroUsize::new(settings.page_items).unwrap(), NonZeroUsize::new(settings.message_bytes).unwrap());
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(diagnostics.clone()))?;
    let server = services::serve_with_diagnostics(endpoint, &root, host_settings, settings, diagnostics).await?;
    tracing::info!("Execution Host Is Ready");
    println!("http://{}", server.endpoint());
    std::io::stdout().flush()?;
    let mut input = [0u8; 1];
    while tokio::io::stdin().read(&mut input).await? != 0 {}
    server.close().await?;
    Ok(())
}

/* === Tests === */
