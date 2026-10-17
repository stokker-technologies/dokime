/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Show Live Operation Provider Contracts

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
    #[arg(help = "Operation Identifier From `operation list`.")]
    operation: String,
    #[arg(long, help = "Select The Discovered Provider Owner.")]
    owner: Option<String>,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit Compact Provider Contracts As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let client = args.connection.connect().await?;
    let result = client.operations().await;
    client.close_and_wait().await;
    let providers: Vec<_> = result?
        .into_iter()
        .filter(|entry| {
            entry.contract.operation().id.as_str() == args.operation
                && args.owner.as_deref().is_none_or(|owner| entry.owner.as_str() == owner)
        })
        .collect();
    if providers.is_empty() {
        return Err(dokime_client::operations::Error::Unknown(args.operation).into());
    }
    let output = if args.json { serde_json::to_string(&providers)? } else { serde_json::to_string_pretty(&providers)? };
    println!("{output}");
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
