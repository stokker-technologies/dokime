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
    env::{self, consts::EXE_SUFFIX},
    future,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

// Third Party Imports
use tempfile::Builder;
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    time,
};

// Local Imports
use crate::{Result, error::Error, settings::Settings};
use dokime_packaging::bundle::{self, DevelopmentInput};
use dokime_runtime::{PluginProcess, STOP_GRACE, process, services};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Bundle File or Plugin Folder Containing dokime.toml")]
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
    let mut desktop = None;
    let mut stdin = tokio::io::stdin();
    let mut buffer = [0; 1];

    // Start Through the Server Command and the Existing Plugin Commands
    let result: Result<()> = async {
        let endpoint = {
            let activate = async {
                if input == DevelopmentInput::Project {
                    super::plugin::project::prepare(&plugin).await?;
                }
                let started = super::server::start(root.path(), Some("127.0.0.1:0".into()), false, None, &settings, true).await?;
                let endpoint = format!("http://{}", started.endpoint());
                server = Some(started);
                match input {
                    DevelopmentInput::Bundle => {
                        super::plugin::install::attach(&plugin, Some(endpoint.clone()), &settings, None).await?;
                    }
                    DevelopmentInput::Project => {
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
        if let Some(executable) = desktop_executable(&env::current_exe()?) {
            let mut command = process::command(executable);
            command
                .args(["--endpoint", &endpoint, "--shutdown-stdin"])
                .env("DOKIME_CREDENTIAL_NAMESPACE", &settings.credential_namespace)
                .env_remove("DOKIME_REMOTE")
                .stdin(Stdio::piped());
            desktop = Some(Command::from(command).kill_on_drop(true).spawn()?);
        } else {
            eprintln!("Desktop Unavailable: No Existing CLI Desktop Locator; Started the Host Only");
        }
        io::stdout().flush()?;
        loop {
            tokio::select! {
                result = tokio::signal::ctrl_c() => { result?; break; }
                result = stdin.read(&mut buffer) => { if result? == 0 { break; } }
                result = async {
                    match desktop.as_mut() {
                        Some(desktop) => desktop.wait().await,
                        None => future::pending().await,
                    }
                } => {
                    let status = result?;
                    if !status.success() {
                        return Err(format!("Develop Plugin: Desktop Exited with {status}; Inspect Its Console Logs").into());
                    }
                    break;
                }
            }
        }
        Ok(())
    }
    .await;

    // Stop Every Started Owner Before Removing Its Credentials and Root
    let desktop_closed = match desktop.as_mut() {
        Some(desktop) => close_desktop(desktop).await.map_err(dokime_runtime::Error::from),
        None => Ok(()),
    };
    drop(desktop);
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
    if let Err(error) = desktop_closed.and(closed).and(stopped).and(removed) {
        let retained = root.keep();
        return Err(
            format!("Develop Plugin: {result:?}; Cleanup Failed: {error}; Inspect the Retained Root at {}", retained.display()).into()
        );
    }
    root.close()?;
    result
}

/* === Private Functions === */

fn desktop_executable(cli: &Path) -> Option<PathBuf> {
    let desktop = cli.parent()?.join(format!("dokime-gui{EXE_SUFFIX}"));
    desktop.is_file().then_some(desktop)
}

async fn close_desktop(desktop: &mut Child) -> io::Result<()> {
    drop(desktop.stdin.take());
    match time::timeout(STOP_GRACE, desktop.wait()).await {
        Ok(result) => {
            result?;
        }
        Err(_) => {
            desktop.kill().await?;
        }
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{env::consts::EXE_SUFFIX, fs};

    use super::desktop_executable;
    use crate::Result;

    #[test]
    fn desktop_lookup_uses_only_the_cli_sibling() -> Result<()> {
        let root = tempfile::tempdir()?;
        let bin = root.path().join("bin");
        fs::create_dir(&bin)?;
        let cli = bin.join(format!("dokime{EXE_SUFFIX}"));
        fs::write(root.path().join(format!("dokime-gui{EXE_SUFFIX}")), b"other location")?;
        assert!(desktop_executable(&cli).is_none());
        let sibling = bin.join(format!("dokime-gui{EXE_SUFFIX}"));
        fs::write(&sibling, b"desktop fixture")?;
        assert_eq!(desktop_executable(&cli), Some(sibling.clone()));
        fs::remove_file(sibling)?;
        assert!(desktop_executable(&cli).is_none());
        Ok(())
    }
}
