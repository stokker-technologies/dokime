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
use std::collections::BTreeSet;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::time::Duration;

// Third Party Imports
use dokime::harmos::plugin::inventory::CarrierSection;
use harmos_packaging::bundle::{Bundle, Limits};

// Local Imports
use super::disclosure::{EnvironmentReference, environment_references};
use crate::terminal::Terminal;
use crate::{Error, Result};
use dokime::client::Client;
use dokime::client::contracts::plugins;
use dokime::domain::model::plugins::wire;

/* === Constants === */

const CONSENT_DEADLINE: Duration = Duration::from_secs(10);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[command(about = "Stage A Plugin Bundle On A Backend, Review It, And Authorize Next-Boot Activation.")]
pub(crate) struct Args {
    bundle: PathBuf,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Authorize The Verified Plugin Without An Interactive Confirmation.")]
    yes: bool,
}

struct Review {
    bundle: Bundle,
    digest: String,
    owners: BTreeSet<String>,
    references: BTreeSet<EnvironmentReference>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> Result<()> {
    if !args.yes && !std::io::stdin().is_terminal() {
        return Err(Error::Input("interactive consent requires a terminal; use --yes for automation".into()));
    }

    // Verify The Bundle Locally
    let limits = Limits::default();
    let bytes = dokime::harmos::filesystem::input::read(&args.bundle, limits.archive_bytes)?;
    let review = review(&bytes, limits)?;

    // Stage It On The Backend
    let client = args.connection.connect().await?;
    let endpoint = args.connection.resolve()?.selection.target.endpoint().to_owned();
    let staged = client.plugins_stage_bundle(bytes, review.digest.clone(), |_, _| {}).await.map_err(|error| {
        Error::Setup(format!(
            "staging failed: {error}. If the response was lost, inspect Plugins on {endpoint} before retrying; the stage may exist."
        ))
    })?;
    let result = async {
        verify(&staged, &review, &endpoint)?;

        // Ask For Consent
        let terminal = Terminal::default();
        let identity = &review.bundle.manifest().identity;
        terminal.field("Host", &endpoint);
        terminal.field("Plugin", format!("{} {}", identity.name, identity.version));
        terminal.field("Description", &identity.description);
        terminal.field("Executables", review.owners.iter().cloned().collect::<Vec<_>>().join(", "));
        terminal.field("Digest", &review.digest);
        terminal.field("Installation", &staged.install_id);
        disclose(&mut std::io::stderr(), &review.references)?;
        let consent = args.yes || confirm()?;

        complete(&client, &endpoint, &staged.install_id, &review.digest, consent).await
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */

fn review(bytes: &[u8], limits: Limits) -> Result<Review> {
    let bundle = Bundle::inspect(bytes, limits).map_err(|error| Error::Input(format!("invalid plugin bundle: {error}")))?;
    let digest = dokime::client::plugin_bundle_digest(bytes);
    let references = environment_references(&bundle)?;
    // A guest declaration was already refused above; this only skips assets.
    let owners = bundle
        .manifest()
        .files
        .iter()
        .filter_map(|file| match bundle.declaration(&file.path) {
            Some(value) if matches!(value.carrier, CarrierSection::Sidecar { .. }) => Some(value.id.clone()),
            Some(_) | None => None,
        })
        .collect();

    Ok(Review { bundle, digest, owners, references })
}

fn verify(staged: &wire::PluginInstallation, review: &Review, endpoint: &str) -> Result<()> {
    let identity = &review.bundle.manifest().identity;
    let verified = staged.metadata.as_ref().is_some_and(|metadata| {
        metadata.identity.as_ref().is_some_and(|staged| {
            staged.name == identity.name && staged.version == identity.version.to_string() && staged.description == identity.description
        }) && metadata.owners.iter().cloned().collect::<BTreeSet<_>>() == review.owners
            && metadata
                .environment_references
                .iter()
                .map(|reference| EnvironmentReference {
                    owner: reference.owner.clone(),
                    setting: reference.setting.clone(),
                    variable: reference.variable.clone(),
                })
                .collect::<BTreeSet<_>>()
                == review.references
    });
    let uuid = staged.install_id.len() == 36
        && staged
            .install_id
            .bytes()
            .enumerate()
            .all(|(index, byte)| if [8, 13, 18, 23].contains(&index) { byte == b'-' } else { byte.is_ascii_hexdigit() });
    if !uuid
        || staged.state != wire::PluginInstallationState::Staged
        || staged.sha256 != review.digest
        || staged.error.is_some()
        || !verified
    {
        return Err(Error::Setup(format!(
            "host stage {:?} did not match the verified bundle; no consent sent. Review this installation on {endpoint}.",
            staged.install_id
        )));
    }
    Ok(())
}

fn disclose(output: &mut impl Write, references: &BTreeSet<EnvironmentReference>) -> Result<()> {
    if references.is_empty() {
        return Ok(());
    }
    writeln!(output, "Environment references (names only; values are never displayed):")?;
    for reference in references {
        // Quoting escapes terminal control characters in untrusted labels.
        writeln!(output, "  Owner: {:?}; setting: {:?}; variable: {:?}", reference.owner, reference.setting, reference.variable)?;
    }
    Ok(())
}

fn confirm() -> Result<bool> {
    eprint!("Authorize these executables for the next host start? [y/N] ");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"))
}

async fn complete(client: &Client, endpoint: &str, id: &str, digest: &str, consent: bool) -> Result<()> {
    let client = client.clone().with_timeout(CONSENT_DEADLINE);
    if !consent {
        let request = plugins::Remove { install_id: id.into() };
        let removed = client.plugins_remove(request).await;
        if !matches!(&removed, Ok(reply) if reply.installation.is_none()) {
            return Err(Error::Setup(format!(
                "stage {id} was not consented, but its cleanup was not confirmed. Retry removal in Plugins on {endpoint}."
            )));
        }
        Terminal::default().status(format!("Discarded staged installation {id}."));
        return Ok(());
    }

    // Once consent is attempted, never delete or blindly retry an uncertain result.
    let request = plugins::Consent { install_id: id.into(), expected_sha256: digest.into() };
    match client.plugins_consent(request).await {
        Ok(accepted)
            if accepted.install_id == id && accepted.sha256 == digest && accepted.state == wire::PluginInstallationState::Installed =>
        {
            Terminal::default().success(format!("Authorized installation {id}; restart the host to activate it."));
            Ok(())
        }
        outcome => Err(Error::Setup(format!(
            "consent for installation {id} was not confirmed: {}. Keep this UUID and inspect Plugins on {endpoint}; it may be authorized and awaiting restart. No removal or retry was attempted.",
            match outcome {
                Err(error) => error.to_string(),
                Ok(_) => "unexpected host response".into(),
            }
        ))),
    }
}

/* === Tests === */
