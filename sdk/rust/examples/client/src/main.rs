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

// Third Party Imports

// Local Imports
use dokime::{Client, ClientSettings, StorageTransferSettings};
use std::{error::Error, num::NonZeroUsize};
use tokio::runtime::Builder;
use tonic::transport::Endpoint;

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
    let endpoint = std::env::args().nth(1).ok_or("Connect Client: Supply the Host Endpoint")?;
    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(async {
        let channel = Endpoint::from_shared(endpoint)?.connect().await?;
        let settings = ClientSettings::for_storage(
            StorageTransferSettings { chunk_bytes: Some(1024 * 1024) },
            NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
        )?;
        let client = Client::from_channel(channel, settings)?;
        let plugins = client.plugins.list().await?;
        for plugin in plugins.entries {
            println!("{}", plugin.name);
        }
        Ok(())
    })
}

/* === Tests === */
