/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Execution Evidence With Explicit Interrupt Ownership

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
    input: super::super::evidence::Args,

    #[command(flatten)]
    recovery: super::recovery::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    super::super::evidence::run(args.input, dokime::domain::model::exports::EvidenceSubject::Execution, Some(args.recovery)).await
}

/* === Private Functions === */

/* === Tests === */
