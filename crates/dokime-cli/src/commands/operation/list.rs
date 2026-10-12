/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

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
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit The Full Operation Contracts As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let client = args.connection.connect().await?;
    let result = client.operations();
    client.close_and_wait().await;
    let operations = result?;
    if args.json {
        println!("{}", serde_json::to_string(&operations)?);
    } else {
        for operation in operations {
            let definition = operation.contract.operation();
            println!("{}\t{}", definition.id, definition.description);
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
