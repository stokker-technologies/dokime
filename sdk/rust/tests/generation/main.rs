/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compile the Generated Client and Plugin Barrels as a Consumer.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::client::Client;
use dokime::plugin::Context;
use dokime::{FileReference, FileSource};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Compiler Story: No Host, Runtime, Storage or Fixtures Are Required.
pub async fn generated_surface(client: &Client, context: &Context) -> Result<(), Box<dyn std::error::Error>> {
    let _ = client.plugins.list().await?;
    let host = context.host();
    let _ = host.files.capture(FileSource::default(), String::new()).await?;
    let _ = host.content.artifacts.create(FileReference::default(), "capture.bin".into(), "application/octet-stream".into()).await?;
    let _ = host.evidence.statistics(String::new(), String::new()).await?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
