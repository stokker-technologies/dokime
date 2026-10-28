/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Backend Profile Use

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    config: super::Config,

    #[arg(long, help = "Saved Backend Profile Identifier.")]
    identifier: String,
}

pub(super) fn run(args: Args) -> crate::Result<()> {
    args.config.store()?.update(|document| {
        dokime::client::profiles::get(document, &args.identifier)?;
        document.default = Some(args.identifier);
        Ok(())
    })?;
    Ok(())
}
