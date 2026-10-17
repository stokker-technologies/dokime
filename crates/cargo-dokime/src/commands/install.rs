/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod transport;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::BTreeSet,
    io::{IsTerminal, Write},
    path::PathBuf,
};
// Third Party Imports
use harmos::bundle::{Bundle, Limits};
use harmos::plugin::inventory::CarrierSection;
use sha2::{Digest, Sha256};
// Local Imports
use super::environment::{EnvironmentReference, requested_environment_references};
use crate::error::Error;
use dokime_client::contracts::plugins;
use dokime_cli::terminal::Terminal;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(clap::Args)]
#[command(
    about = "Stage a bundle on a running Dokime host, review it, and authorize next-boot activation"
)]
pub struct Args {
    bundle: PathBuf,
    #[arg(long, default_value = "http://127.0.0.1:8765")]
    endpoint: String,
    #[arg(long, help = "File containing the backend bearer token.")]
    token_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Additional PEM certificate authority for an HTTPS backend."
    )]
    ca_cert: Option<PathBuf>,
    #[arg(
        long,
        help = "Authorize the verified plugin without an interactive confirmation."
    )]
    yes: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub async fn run(args: Args) -> Result<(), Error> {
    if !args.yes && !std::io::stdin().is_terminal() {
        return Err(Error::Invalid(
            "interactive consent requires a terminal; use --yes for automation".into(),
        ));
    }
    let base = transport::endpoint(&args.endpoint)?;
    let limits = Limits::default();
    let bytes = dokime_packaging::bundle::read(&args.bundle)?;
    let bundle = Bundle::inspect(&bytes, limits)?;
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let environment_references = requested_environment_references(&bundle)?;
    let owners = bundle
        .manifest()
        .files
        .iter()
        // A guest declaration was already refused above; this only skips assets.
        .filter_map(|file| match bundle.declaration(&file.path) {
            Some(value) if matches!(value.carrier, CarrierSection::Sidecar { .. }) => {
                Some(value.id.clone())
            }
            Some(_) | None => None,
        })
        .collect::<BTreeSet<_>>();
    let mut client =
        transport::Client::connect(&base, args.token_file.as_deref(), args.ca_cert.as_deref())
            .await?;
    let entry = client.stage(bytes, digest.clone()).await
        .map_err(|error| Error::Invalid(format!("staging failed: {error}. If the response was lost, inspect Plugins on {base} before retrying; the stage may exist.")))?;
    validate_staged(
        &entry,
        &bundle,
        &owners,
        &environment_references,
        &digest,
        &base,
    )?;
    let id = &entry.install_id;
    let terminal = Terminal::default();
    terminal.field("Host", &base);
    terminal.field(
        "Plugin",
        format!(
            "{} {}",
            bundle.manifest().identity.name,
            bundle.manifest().identity.version
        ),
    );
    terminal.field("Description", &bundle.manifest().identity.description);
    terminal.field(
        "Executables",
        owners.into_iter().collect::<Vec<_>>().join(", "),
    );
    terminal.field("Digest", &digest);
    terminal.field("Installation", id);
    disclose_environment_references(&mut std::io::stderr(), &environment_references)?;
    let consent = if args.yes {
        true
    } else {
        eprint!("Authorize these executables for the next host start? [y/N] ");
        std::io::stderr().flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
    };
    complete(&mut client, &base, id, &digest, consent).await
}

/* === Private Functions === */
fn disclose_environment_references(
    output: &mut impl Write,
    references: &BTreeSet<EnvironmentReference>,
) -> Result<(), Error> {
    if references.is_empty() {
        return Ok(());
    }
    writeln!(
        output,
        "Environment references (names only; values are never displayed):"
    )?;
    for reference in references {
        // Quoting escapes terminal control characters in untrusted labels.
        writeln!(
            output,
            "  Owner: {:?}; setting: {:?}; variable: {:?}",
            reference.owner, reference.setting, reference.variable
        )?;
    }
    Ok(())
}

fn validate_staged(
    entry: &plugins::PluginInstallation,
    bundle: &Bundle,
    owners: &BTreeSet<String>,
    environment_references: &BTreeSet<EnvironmentReference>,
    digest: &str,
    base: &url::Url,
) -> Result<(), Error> {
    validate_id(&entry.install_id)?;
    let verified = entry.metadata.as_ref().is_some_and(|metadata| {
        metadata.identity.as_ref().is_some_and(|identity| {
            identity.name == bundle.manifest().identity.name
                && identity.version == bundle.manifest().identity.version.to_string()
                && identity.description == bundle.manifest().identity.description
        }) && metadata.owners.iter().cloned().collect::<BTreeSet<_>>() == *owners
            && metadata
                .environment_references
                .iter()
                .map(|reference| EnvironmentReference {
                    owner: reference.owner.clone(),
                    setting: reference.setting.clone(),
                    variable: reference.variable.clone(),
                })
                .collect::<BTreeSet<_>>()
                == *environment_references
    });
    if entry.state != plugins::PluginInstallationState::Staged
        || entry.sha256 != digest
        || entry.error.is_some()
        || !verified
    {
        return Err(Error::Invalid(format!(
            "host stage {} did not match the verified bundle; no consent sent. Review this UUID on {base}.",
            entry.install_id
        )));
    }
    Ok(())
}

async fn complete(
    client: &mut transport::Client,
    base: &url::Url,
    id: &str,
    digest: &str,
    consent: bool,
) -> Result<(), Error> {
    if !consent {
        client.remove(id).await.map_err(|error| Error::Invalid(format!("stage {id} was not consented, but cleanup failed: {error}. Retry removal in Plugins.")))?;
        Terminal::default().status(format!("Discarded staged installation {id}."));
        return Ok(());
    }
    // Once consent is attempted, never delete or blindly retry an uncertain result.
    match client.consent(id, digest).await {
        Ok(accepted)
            if accepted.install_id == id
                && accepted.sha256 == digest
                && accepted.state == plugins::PluginInstallationState::Installed =>
        {
            Terminal::default().success(format!(
                "Authorized installation {id}; restart the host to activate it."
            ));
            Ok(())
        }
        outcome => Err(Error::Invalid(format!(
            "consent for installation {id} was not confirmed: {}. Keep this UUID and inspect Plugins on {base}; it may be authorized and awaiting restart. No removal or retry was attempted.",
            match outcome {
                Err(error) => error.to_string(),
                Ok(_) => "unexpected host response".into(),
            }
        ))),
    }
}

fn validate_id(id: &str) -> Result<(), Error> {
    if id.len() != 36
        || !id.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
    {
        return Err(Error::Invalid(
            "host returned an invalid installation UUID; no consent sent".into(),
        ));
    }
    Ok(())
}

/* === Tests === */
