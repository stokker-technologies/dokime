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
use std::{io::Write, path::PathBuf};

// Third Party Imports
use tokio::io::AsyncReadExt;

// Local Imports

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

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    // Resolve Every Environment Read Before the Host Starts
    let environment = crate::environment::Environment::read(args.env_file.as_deref())?;
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .parse(environment.get("RUST_LOG").unwrap_or_default())
        .map_err(|error| crate::error::Error::Invalid(format!("Configure Diagnostic Logging: {error}")))?;
    let endpoint = endpoint.ok_or_else(|| crate::error::Error::Invalid("Supply --endpoint HOST:PORT or Configure endpoint".into()))?;
    let endpoint = endpoint.strip_prefix("http://").unwrap_or(&endpoint).parse()?;
    let storage = args
        .storage
        .or(settings.storage.clone())
        .ok_or_else(|| crate::error::Error::Invalid("Supply --storage PATH or Configure storage".into()))?;
    let server =
        Box::pin(dokime_runtime::services::serve(endpoint, &storage, settings.host(args.controlled_clock), settings.services())).await?;
    use tracing_subscriber::prelude::*;
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(filter).with(server.status().layer()))
        .map_err(|error| crate::error::Error::Invalid(format!("Start Diagnostic Logging: {error}")))?;
    println!("http://{}", server.endpoint());
    std::io::stdout().flush()?;
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

/* === Private Functions === */

/* === Tests === */
