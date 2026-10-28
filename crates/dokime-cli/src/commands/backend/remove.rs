/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Backend Profile Remove

use dokime::client::{
    profiles::{self, Secret},
    secrets::{KeyringStore, SecretStore},
};

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    config: super::Config,

    #[arg(long, help = "Saved Backend Profile Identifier.")]
    identifier: String,
}

pub(super) fn run(args: Args) -> crate::Result<()> {
    let removed = args.config.store()?.update(|document| {
        let removed = profiles::selection(profiles::get(document, &args.identifier)?)?;
        document.profiles.retain(|profile| profile.identifier != args.identifier);
        if document.default.as_deref() == Some(&args.identifier) {
            document.default = None;
        }
        Ok(removed)
    })?;
    if removed.secret == Secret::Keyring {
        KeyringStore.remove(&removed.credential_key())?;
    }
    Ok(())
}
