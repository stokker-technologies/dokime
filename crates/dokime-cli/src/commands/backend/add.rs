/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Backend Profile Add

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::client::profiles::{Error, Profile, Secret};

/* === Constants === */

const CA_BYTES: u64 = 1_048_576;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    config: super::Config,

    #[arg(help = "Saved Backend Profile Name.")]
    name: String,

    #[arg(long, help = "Backend HTTP(S) Origin.")]
    endpoint: String,

    #[arg(long, help = "Require This Backend Identity During Connection.")]
    expected_backend: Option<String>,

    #[arg(long, help = "Read An Additional PEM CA Certificate From This Local File.")]
    ca: Option<std::path::PathBuf>,

    #[arg(long, help = "Resolve Secrets From The OS Store On Connection.")]
    keyring: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn run(args: Args) -> crate::Result<()> {
    let ca_certificate_pem = args
        .ca
        .as_deref()
        .map(|path| {
            let bytes = dokime::harmos::filesystem::input::read(path, CA_BYTES as usize)?;
            String::from_utf8(bytes).map_err(|_| crate::Error::Input("CA certificate must be UTF-8 PEM".into()))
        })
        .transpose()?;
    let profile = Profile {
        name: args.name,
        endpoint: args.endpoint,
        expected_backend_id: args.expected_backend,
        ca_certificate_pem,
        secret: if args.keyring { Secret::Keyring } else { Secret::None },
    };
    dokime::client::profiles::selection(&profile)?;
    args.config.store()?.update(move |document| {
        if document.profiles.iter().any(|existing| existing.name == profile.name) {
            return Err(Error::Invalid("profile already exists".into()));
        }
        document.profiles.push(profile);
        Ok(())
    })?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
