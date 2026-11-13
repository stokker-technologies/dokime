/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Connect Through and Export Portable Pairing Files

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

// Third Party Imports
use dokime::client::v1::{self, access};
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

const PAIRING_BYTES: usize = 1_048_576;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct File {
    version: u32,
    endpoints: Vec<String>,
    backend_identifier: String,
    label: String,
    ca_certificate_pem: String,
    access_key: String,
}

#[derive(clap::Args)]
pub(crate) struct ExportArgs {
    #[arg(long)]
    out: PathBuf,

    #[arg(long)]
    label: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    #[command(subcommand)]
    Pairing(PairingCommand),
}

#[derive(clap::Subcommand)]
pub(crate) enum PairingCommand {
    Export(ExportArgs),
}

/* === Traits === */

/* === Implementations === */

impl File {
    pub fn read(path: &Path) -> crate::Result<Self> {
        if path.extension().and_then(|value| value.to_str()) != Some("dokime-remote") {
            return Err(crate::error::Error::Invalid("Select a .dokime-remote Pairing File".into()));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?.take(PAIRING_BYTES as u64 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > PAIRING_BYTES {
            return Err(crate::error::Error::Capacity("Pairing File Exceeds 1 MiB; Export a New Pairing".into()));
        }
        let file: Self = serde_json::from_slice(&bytes)
            .map_err(|_| crate::error::Error::Invalid("Invalid Pairing File; Export a New Version 1 Pairing".into()))?;
        if file.version != 1
            || file.endpoints.is_empty()
            || file.endpoints.len() > 32
            || file.backend_identifier.is_empty()
            || file.backend_identifier.len() > 128
            || file.backend_identifier.chars().any(char::is_whitespace)
            || file.label.trim().is_empty()
            || file.label.len() > 256
            || file.label.chars().any(char::is_control)
            || file.access_key.is_empty()
            || file.access_key.chars().any(char::is_whitespace)
        {
            return Err(crate::error::Error::Invalid("Invalid Pairing Identity, Label, Endpoints or Key; Export a New Pairing".into()));
        }
        file.authorization()?;
        for (index, endpoint) in file.endpoints.iter().enumerate() {
            let value = tonic::transport::Endpoint::from_shared(endpoint.clone())?;
            let uri = value.uri();
            if uri.scheme_str() != Some("https")
                || uri.host().is_none_or(|host| host.contains('*'))
                || uri.authority().is_none_or(|authority| authority.as_str().contains('@'))
                || uri.port_u16().is_none_or(|port| port == 0)
                || uri.path_and_query().is_some_and(|path| path.as_str() != "/")
                || file.endpoints[..index].contains(endpoint)
            {
                return Err(crate::error::Error::Invalid("Pairing Endpoints Must Be Distinct HTTPS Origins With Explicit Ports".into()));
            }
        }
        Ok(file)
    }
    pub fn authorization(&self) -> crate::Result<tonic::metadata::MetadataValue<tonic::metadata::Ascii>> {
        let mut value: tonic::metadata::MetadataValue<tonic::metadata::Ascii> = format!("Bearer {}", self.access_key)
            .parse()
            .map_err(|_| crate::error::Error::Invalid("Invalid Pairing Key; Export a New Pairing".into()))?;
        value.set_sensitive(true);
        Ok(value)
    }
    pub async fn channel(&self, settings: &crate::settings::Settings, streaming: bool) -> crate::Result<tonic::transport::Channel> {
        let timeout = if streaming { settings.watch_timeout_s } else { settings.mutation_timeout_s.max(settings.read_timeout_s) };
        let mut failures = Vec::new();
        for endpoint in &self.endpoints {
            let result: crate::Result<_> = async {
                let tls = tonic::transport::ClientTlsConfig::new()
                    .ca_certificate(tonic::transport::Certificate::from_pem(&self.ca_certificate_pem));
                let channel = tonic::transport::Endpoint::from_shared(endpoint.clone())?
                    .tls_config(tls)?
                    .connect_timeout(Duration::from_secs(settings.connect_timeout_s))
                    .timeout(Duration::from_secs(timeout))
                    .connect()
                    .await?;
                let client = dokime::Client::from_channel(channel.clone(), settings.client())?;
                let mut request = tonic::Request::new(v1::StatusGetRequest {});
                request.metadata_mut().insert("authorization", self.authorization()?);
                let reply = client.status.get(request).await?.into_inner();
                let identity = reply.status.and_then(|status| status.identity).ok_or("Host Did Not Report Its Identity")?;
                if identity.host_id != self.backend_identifier {
                    return Err("Pairing Backend Identity Differs; Export a Pairing From the Intended Host".into());
                }
                Ok(channel)
            }
            .await;
            match result {
                Ok(channel) => return Ok(channel),
                Err(error) => failures.push(format!("{endpoint}: {error}")),
            }
        }
        Err(format!("Connect Pairing: {}; Check the Host Listener or Export a New Pairing", failures.join("; ")).into())
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let Command::Pairing(PairingCommand::Export(args)) = command;
    if args.out.extension().and_then(|value| value.to_str()) != Some("dokime-remote") {
        return Err(crate::error::Error::Invalid("Export to a .dokime-remote File".into()));
    }
    if args.out.exists() {
        return Err(crate::error::Error::Invalid("Choose a New Pairing Destination; the File Already Exists".into()));
    }
    let parent = args.out.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut output = tempfile::NamedTempFile::new_in(parent)?;
    let channel = settings.channel(endpoint, false).await?;
    let client = dokime::Client::from_channel(channel, settings.client())?;

    // Enable the Saved Listener Before Requesting Remote Access
    let current = client.access.listener.get(settings.request(access::ListenerGetRequest {})).await?.into_inner();
    let mut value = current.value.ok_or("Host Did Not Report Listener Settings")?;
    if !value.enabled {
        value.enabled = true;
        client
            .access
            .listener
            .set(settings.request(access::ListenerSetRequest {
                value: Some(value),
                expected_revision: current.revision,
            }))
            .await?;
    }
    let identity = client
        .status
        .get(settings.request(v1::StatusGetRequest {}))
        .await?
        .into_inner()
        .status
        .and_then(|status| status.identity)
        .ok_or("Host Did Not Report Its Identity")?;
    let created = client.access.pairings.create(settings.request(access::PairingsCreateRequest { label: args.label })).await?.into_inner();
    let pairing = created.pairing.as_ref().ok_or("Host Did Not Report the Created Pairing")?;
    let file = File {
        version: 1,
        endpoints: vec![created.endpoint],
        backend_identifier: identity.host_id,
        label: pairing.label.clone(),
        ca_certificate_pem: created.pinned_certificate_pem,
        access_key: created.credential,
    };

    // Publish Secret Bytes Atomically; Revoke the New Pairing if Publication Fails
    let result: crate::Result<()> = (|| {
        let bytes = serde_json::to_vec(&file)?;
        if bytes.len() > PAIRING_BYTES {
            return Err(crate::error::Error::Capacity("Exported Pairing Exceeds 1 MiB".into()));
        }
        output.write_all(&bytes)?;
        output.as_file().sync_all()?;
        output.persist_noclobber(&args.out)?;
        Ok(())
    })();
    if let Err(error) = result {
        client.access.pairings.delete(settings.request(access::PairingsDeleteRequest { name: pairing.id.clone() })).await?;
        return Err(error);
    }
    if args.json {
        println!("{}", serde_json::json!({"path": args.out, "pairing_id": pairing.id}));
    } else {
        println!("Exported Pairing {} to {}", pairing.id, args.out.display());
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
