/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Serve a Built Plugin for Development

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Write, path::PathBuf};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    executable: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let mut process = dokime_runtime_next::PluginProcess::start(&args.executable, true, &settings.placement()).await?;
    let result: crate::Result<()> = async {
        if endpoint.is_some() {
            let channel = settings.channel(endpoint, false).await?;
            let mut client = dokime::v1::plugins_client::PluginsClient::with_interceptor(channel, settings.interceptor());
            client.add(dokime::v1::AddRemotePlugin { url: process.endpoint().into() }).await?;
        }
        println!("{}", process.endpoint());
        std::io::stdout().flush()?;
        use tokio::io::AsyncReadExt;
        let mut input = [0u8; 1];
        let mut stdin = tokio::io::stdin();
        loop {
            tokio::select! {
                result = tokio::signal::ctrl_c() => { result?; break; }
                result = stdin.read(&mut input) => { if result? == 0 { break; } }
            }
        }
        Ok(())
    }
    .await;
    let closed = process.close().await;
    result?;
    closed?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
