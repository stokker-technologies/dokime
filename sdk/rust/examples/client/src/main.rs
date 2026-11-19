/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Public SDK Consumer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env,
    error::Error,
    io::{BufRead, BufReader},
    num::NonZeroUsize,
    process::{Command, Stdio},
};

// Third Party Imports
use tokio::runtime::Builder;
use tonic::transport::Endpoint;

// Local Imports
use dokime::client::{PluginPlacement, PluginRemote, PluginState, PluginValue, PluginsWatchOptions, plugin_placement};
use dokime::{Client, ClientSettings, StorageTransferSettings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args().skip(1);
    let endpoint = arguments.next().ok_or("Connect Client: Supply the Host Endpoint and Plugin Executable")?;
    let executable = arguments.next().ok_or("Start Plugin: Supply the Capture Plugin Executable")?;

    // Start the Sidecar
    let mut child = Command::new(executable).stdout(Stdio::piped()).spawn()?;
    let result = (|| -> Result<(), Box<dyn Error>> {
        let output = child.stdout.take().ok_or("Start Plugin: Enable Its Readiness Output")?;
        let mut output = BufReader::new(output);
        let mut readiness = String::new();
        output.read_line(&mut readiness)?;
        let address = readiness.split_whitespace().last().ok_or("Start Plugin: Read Its Listening Address")?;
        let url = format!("http://{address}");

        // Connect the Client
        let runtime = Builder::new_multi_thread().enable_all().build()?;
        runtime.block_on(async {
            let channel = Endpoint::from_shared(endpoint)?.connect().await?;
            let settings = ClientSettings::for_storage(
                StorageTransferSettings { chunk_bytes: Some(1024 * 1024) },
                NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
            )?;
            let client = Client::from_channel(channel, settings)?;

            // Attach the Plugin
            let placement = PluginPlacement {
                kind: Some(plugin_placement::Kind::Remote(PluginRemote { url })),
            };
            let created = client.plugins.create("example", placement, PluginValue { enabled: true }).await?;
            let entry = created.entry.ok_or("Attach Plugin: Read the Created Entry")?;
            let result = async {
                let mut watching = client
                    .plugins
                    .watch_with(PluginsWatchOptions {
                        name: Some(entry.name.clone()),
                        ..Default::default()
                    })
                    .await?;
                loop {
                    let current = client.plugins.get(&entry.name).await?.entry.ok_or("Attach Plugin: Read Its Entry")?;
                    if current.failure.is_some() {
                        return Err("Attach Plugin: Resolve Its Reported Failure".into());
                    }
                    if current.state == PluginState::Ready as i32 {
                        break;
                    }
                    watching.next().await?.ok_or("Attach Plugin: Reconnect After Its Watch Closed")?;
                }

                // Read the Authored Suite
                for suite in client.suites.list(&entry.name).await?.values {
                    let metadata = suite.metadata.ok_or("Read Suite: Supply Its Metadata")?;
                    println!("{}", metadata.identifier);
                }
                Ok::<(), Box<dyn Error>>(())
            }
            .await;

            // Detach the Plugin
            let current = client.plugins.get(&entry.name).await?.entry.ok_or("Detach Plugin: Read Its Entry")?;
            client.plugins.delete(current.name, current.revision).await?;
            result
        })
    })();

    // Join the Sidecar
    if child.try_wait()?.is_none() {
        child.kill()?;
    }
    child.wait()?;
    result
}

/* === Tests === */
