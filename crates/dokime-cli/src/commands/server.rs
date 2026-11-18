/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host the Independent Runtime

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

// Third Party Imports
use tokio::io::AsyncReadExt;
use tracing_subscriber::prelude::*;

// Local Imports
use crate::{Result, error::Error, settings::Settings};
use dokime_runtime::services::{self, Server};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    storage: Option<PathBuf>,

    #[arg(long, help = "Use the Controlled Execution Clock for Deterministic Stories")]
    controlled_clock: bool,

    #[arg(long, help = "Read Host Variables From This File; Process Variables Take Precedence")]
    env_file: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: Settings) -> Result<()> {
    let storage =
        args.storage.or(settings.storage.clone()).ok_or_else(|| Error::Invalid("Supply --storage PATH or Configure storage".into()))?;
    let server = start(&storage, endpoint, args.controlled_clock, args.env_file.as_deref(), &settings, false).await?;
    println!("http://{}", server.endpoint());
    io::stdout().flush()?;
    let mut input = [0u8; 1];
    let mut stdin = tokio::io::stdin();
    loop {
        tokio::select! {
            result = tokio::signal::ctrl_c() => { result?; break; }
            result = stdin.read(&mut input) => { if result? == 0 { break; } }
        }
    }
    server.close().await?;
    Ok(())
}

pub(super) async fn start(
    storage: &Path, endpoint: Option<String>, controlled_clock: bool, env_file: Option<&Path>, settings: &Settings, console: bool,
) -> Result<Server> {
    // Resolve Every Environment Read Before the Host Starts
    let environment = crate::environment::Environment::read(env_file)?;
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .parse(environment.get("RUST_LOG").unwrap_or_default())
        .map_err(|error| Error::Invalid(format!("Configure Diagnostic Logging: {error}")))?;
    let endpoint = endpoint.ok_or_else(|| Error::Invalid("Supply --endpoint HOST:PORT or Configure endpoint".into()))?;
    let endpoint = endpoint.strip_prefix("http://").unwrap_or(&endpoint).parse()?;
    let server = Box::pin(services::serve(endpoint, storage, settings.host(controlled_clock)?, settings.services())).await?;
    let console = console.then(|| tracing_subscriber::fmt::layer().with_writer(io::stderr));
    let logging = tracing_subscriber::registry().with(filter).with(console).with(server.status().layer());
    if let Err(error) = tracing::subscriber::set_global_default(logging) {
        server.close().await?;
        return Err(Error::Invalid(format!("Start Diagnostic Logging: {error}")));
    }
    Ok(server)
}

/* === Private Functions === */

/* === Tests === */
