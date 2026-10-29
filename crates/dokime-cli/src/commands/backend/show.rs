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
    config: super::Config,

    #[arg(long, help = "Saved Backend Profile Identifier.")]
    identifier: String,

    #[arg(long, help = "Emit The Saved Profile Configuration As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn run(args: Args) -> crate::Result<()> {
    let document = args.config.store()?.load()?;
    let profile = dokime::client::profiles::get(&document, &args.identifier)?;
    if args.json {
        println!("{}", serde_json::to_string(profile)?);
    } else {
        println!("{}\t{}\t{}", profile.identifier, profile.label, profile.endpoint);
        if let Some(identity) = &profile.expected_backend_id {
            println!("Expected Backend: {identity}");
        }
        println!("Secret Store: {:?}", profile.secret);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
