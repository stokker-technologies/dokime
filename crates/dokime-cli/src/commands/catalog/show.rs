/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Show A Registered Suite Contract

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Suite ID, As The Catalog Lists It.")]
    suite: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit The Suite Definition As Compact JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let client = args.connection.connect().await?;
    let result = client.suite(&args.suite).await;
    client.close_and_wait().await;
    let suite = super::readable(&result?)?;
    let output = if args.json { serde_json::to_string(&suite)? } else { serde_json::to_string_pretty(&suite)? };
    println!("{output}");
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
