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
use dokime::client::plugin_placement::Kind;
use dokime::client::{PluginPlacement, PluginRemote, PluginValue};
use dokime::client::{PluginsCreateOptions, PluginsDeleteOptions};

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
    let mut process = dokime_runtime::PluginProcess::start(&args.executable, true, &settings.placement()).await?;
    let mut registered = None;
    let result: crate::Result<()> = async {
        if endpoint.is_some() || settings.pairing.is_some() {
            let description = process.describe().await?;
            let client = settings.sdk(endpoint).await?;
            let placement = PluginPlacement {
                kind: Some(Kind::Remote(PluginRemote { url: process.endpoint().into() })),
            };
            let created = client
                .plugins
                .create_with(
                    description.id,
                    placement,
                    PluginValue { enabled: true },
                    PluginsCreateOptions {
                        context: settings.context()?,
                        ..Default::default()
                    },
                )
                .await?;
            let entry = created.entry.ok_or("Develop Plugin: Missing Registered Entry")?;
            registered = Some((client, entry));
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
    let removed = match registered {
        Some((client, entry)) => {
            client.plugins.delete_with(entry.name, entry.revision, PluginsDeleteOptions { context: settings.context()? }).await.map(|_| ())
        }
        None => Ok(()),
    };
    let closed = process.close().await;
    result?;
    removed?;
    closed?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
