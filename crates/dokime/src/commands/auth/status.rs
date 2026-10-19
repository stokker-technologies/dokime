/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Authentication And Connection Inspection

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

    #[arg(long, help = "Emit The Result As JSON Without Secret Values.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn run(args: Args) -> crate::Result<()> {
    super::super::status::check(args.connection, args.json).await
}

/* === Private Functions === */

/* === Tests === */
