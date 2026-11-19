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
use dokime::FileSource;
use dokime::client::Client;
use dokime::templates::Context;
use std::error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Compiler Story: No Host, Runtime, Storage or Fixtures Are Required.
pub async fn generated_surface(client: &Client, context: &Context) -> Result<(), Box<dyn error::Error>> {
    let _ = client.plugins().list().await?;
    let _ = context.capture(FileSource::default(), String::new()).await?;
    let _ = context.artifact("capture.bin", Vec::new(), "application/octet-stream").await?;
    let _ = context.statistics(String::new(), String::new(), String::new()).await?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
