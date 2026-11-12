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
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let endpoint = endpoint.ok_or_else(|| crate::error::Error::Invalid("Supply --endpoint HOST:PORT or Configure endpoint".into()))?;
    let endpoint = endpoint.strip_prefix("http://").unwrap_or(&endpoint).parse()?;
    let storage = args
        .storage
        .or(settings.storage.clone())
        .ok_or_else(|| crate::error::Error::Invalid("Supply --storage PATH or Configure storage".into()))?;
    let server =
        dokime_runtime_next::services::serve(endpoint, &storage, settings.host(args.controlled_clock), settings.services()).await?;
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
