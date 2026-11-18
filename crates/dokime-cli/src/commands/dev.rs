/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Start a Disposable Application with One Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{self, Write},
    path::PathBuf,
};

// Third Party Imports
use tempfile::Builder;
use tokio::io::AsyncReadExt;

// Local Imports
use crate::{Result, error::Error, settings::Settings};
use dokime_packaging::bundle::{self, DevelopmentInput};
use dokime_runtime::{PluginProcess, services};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Bundle File or Python Project Folder")]
    plugin: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, mut settings: Settings) -> Result<()> {
    // Classify Before Opening a Disposable Host
    let input = bundle::development_input(&args.plugin)?;
    let plugin = args.plugin.canonicalize()?;
    let root = Builder::new().prefix("dokime-dev-").tempdir()?;
    settings.remote = None;
    settings.pairing = None;
    settings.endpoint = None;
    settings.storage = None;
    settings.credential_namespace =
        root.path().file_name().ok_or("Develop Plugin: Name the Temporary Root")?.to_string_lossy().into_owned();
    let credentials = settings.credentials()?;
    let mut server = None;
    let mut process = None;
    let mut stdin = tokio::io::stdin();
    let mut buffer = [0; 1];

    // Start Through the Server Command and the Existing Plugin Commands
    let result: Result<()> = async {
        let endpoint = {
            let activate = async {
                let started = super::server::start(root.path(), Some("127.0.0.1:0".into()), false, None, &settings, true).await?;
                let endpoint = format!("http://{}", started.endpoint());
                server = Some(started);
                match input {
                    DevelopmentInput::Bundle => {
                        super::plugin::install::attach(&plugin, Some(endpoint.clone()), &settings, None).await?;
                    }
                    DevelopmentInput::PythonProject => {
                        process = Some(PluginProcess::start(&plugin, true, &settings.placement()).await?);
                        let url = process.as_ref().ok_or("Develop Plugin: Retain Its Process")?.endpoint().to_owned();
                        super::plugin::add::attach(url, Some(endpoint.clone()), &settings, None).await?;
                    }
                }
                Ok::<_, Error>(endpoint)
            };
            tokio::pin!(activate);
            loop {
                tokio::select! {
                    result = &mut activate => break result?,
                    result = tokio::signal::ctrl_c() => { result?; return Ok(()); }
                    result = stdin.read(&mut buffer) => { if result? == 0 { return Ok(()); } }
                }
            }
        };
        println!("{endpoint}");
        println!("Data Root: {}", root.path().display());
        eprintln!("Desktop Unavailable: No Existing CLI Desktop Locator; Started the Host Only");
        io::stdout().flush()?;
        loop {
            tokio::select! {
                result = tokio::signal::ctrl_c() => { result?; break; }
                result = stdin.read(&mut buffer) => { if result? == 0 { break; } }
            }
        }
        Ok(())
    }
    .await;

    // Stop Every Started Owner Before Removing Its Credentials and Root
    let closed = match server {
        Some(server) => server.close().await,
        None => Ok(()),
    };
    let stopped = match process.as_mut() {
        Some(process) => process.close().await,
        None => Ok(()),
    };
    drop(process);
    let removed = if root.path().join("backend-id").exists() { services::remove_credentials(root.path(), &credentials) } else { Ok(()) };
    if let Err(error) = closed.and(stopped).and(removed) {
        let retained = root.keep();
        return Err(
            format!("Develop Plugin: {result:?}; Cleanup Failed: {error}; Inspect the Retained Root at {}", retained.display()).into()
        );
    }
    root.close()?;
    result
}

/* === Private Functions === */

/* === Tests === */
