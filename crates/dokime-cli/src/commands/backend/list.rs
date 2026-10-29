/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    config: super::Config,

    #[arg(long, help = "Emit The Saved Profile Configuration As JSON.")]
    json: bool,
}

pub(super) fn run(args: Args) -> crate::Result<()> {
    let document = args.config.store()?.load()?;
    if args.json {
        println!("{}", serde_json::to_string(&document)?);
    } else {
        for profile in &document.profiles {
            let selected = if document.default.as_deref() == Some(&profile.identifier) { "*" } else { " " };
            println!("{selected} {}\t{}\t{}", profile.identifier, profile.label, profile.endpoint);
        }
    }
    Ok(())
}
