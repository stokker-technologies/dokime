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
use std::fs;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

// Third Party Imports
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tonic::metadata::{Ascii, MetadataValue};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};

// Local Imports
use crate::Result;
use crate::error::Error;
use crate::settings::Settings;
use dokime::Client;
use dokime::client::{AccessListenerGetOptions, AccessListenerSetOptions, AccessPairingsCreateOptions, AccessPairingsDeleteOptions};
use dokime::client::{CallOptions, StatusGetOptions};

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
    #[arg(long, default_value = "operator.pair")]
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
    pub fn read(path: &Path) -> Result<Self> {
        if path.extension().and_then(|value| value.to_str()) != Some("pair") {
            return Err(Error::Invalid("Select a .pair Pairing File".into()));
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?.take(PAIRING_BYTES as u64 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > PAIRING_BYTES {
            return Err(Error::Capacity("Pairing File Exceeds 1 MiB; Export a New Pairing".into()));
        }
        let file: Self =
            serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("Invalid Pairing File; Export a New Version 1 Pairing".into()))?;
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
            return Err(Error::Invalid("Invalid Pairing Identity, Label, Endpoints or Key; Export a New Pairing".into()));
        }
        file.authorization()?;
        for (index, endpoint) in file.endpoints.iter().enumerate() {
            let value = Endpoint::from_shared(endpoint.clone())?;
            let uri = value.uri();
            if uri.scheme_str() != Some("https")
                || uri.host().is_none_or(|host| host.contains('*'))
                || uri.authority().is_none_or(|authority| authority.as_str().contains('@'))
                || uri.port_u16().is_none_or(|port| port == 0)
                || uri.path_and_query().is_some_and(|path| path.as_str() != "/")
                || file.endpoints[..index].contains(endpoint)
            {
                return Err(Error::Invalid("Pairing Endpoints Must Be Distinct HTTPS Origins With Explicit Ports".into()));
            }
        }
        Ok(file)
    }
    pub fn authorization(&self) -> Result<MetadataValue<Ascii>> {
        let mut value: MetadataValue<Ascii> = format!("Bearer {}", self.access_key)
            .parse()
            .map_err(|_| Error::Invalid("Invalid Pairing Key; Export a New Pairing".into()))?;
        value.set_sensitive(true);
        Ok(value)
    }
    pub async fn channel(&self, settings: &Settings) -> Result<Channel> {
        let mut failures = Vec::new();
        for endpoint in &self.endpoints {
            let result: Result<_> = async {
                let tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(&self.ca_certificate_pem));
                let channel = Endpoint::from_shared(endpoint.clone())?
                    .tls_config(tls)?
                    .connect_timeout(Duration::from_secs(settings.connect_timeout_s))
                    .connect()
                    .await?;
                let client = Client::from_channel(channel.clone(), settings.client())?;
                let mut context = CallOptions::default();
                context.metadata.insert("authorization", self.authorization()?);
                let reply = client.status.get_with(StatusGetOptions { context }).await?;
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

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: Settings) -> Result<()> {
    let Command::Pairing(PairingCommand::Export(args)) = command;
    if args.out.extension().and_then(|value| value.to_str()) != Some("pair") {
        return Err(Error::Invalid("Export to a .pair File".into()));
    }
    if args.out.exists() {
        return Err(Error::Invalid("Choose a New Pairing Destination; the File Already Exists".into()));
    }
    let parent = args.out.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut output = NamedTempFile::new_in(parent)?;
    let client = settings.sdk(endpoint).await?;

    // Enable the Saved Listener Before Requesting Remote Access
    let current = client.access.listener.get_with(AccessListenerGetOptions { context: settings.context()? }).await?;
    let mut value = current.value.ok_or("Host Did Not Report Listener Settings")?;
    if !value.enabled {
        value.enabled = true;
        client.access.listener.set_with(value, current.revision, AccessListenerSetOptions { context: settings.context()? }).await?;
    }
    let identity = client
        .status
        .get_with(StatusGetOptions { context: settings.context()? })
        .await?
        .status
        .and_then(|status| status.identity)
        .ok_or("Host Did Not Report Its Identity")?;
    let created = client.access.pairings.create_with(args.label, AccessPairingsCreateOptions { context: settings.context()? }).await?;
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
    let result: Result<()> = (|| {
        let bytes = serde_json::to_vec(&file)?;
        if bytes.len() > PAIRING_BYTES {
            return Err(Error::Capacity("Exported Pairing Exceeds 1 MiB".into()));
        }
        output.write_all(&bytes)?;
        output.as_file().sync_all()?;
        output.persist_noclobber(&args.out)?;
        Ok(())
    })();
    if let Err(error) = result {
        client.access.pairings.delete_with(pairing.id.clone(), AccessPairingsDeleteOptions { context: settings.context()? }).await?;
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

#[cfg(test)]
mod tests {
    use tokio::net::TcpSocket;

    use super::File;
    use crate::settings::Settings;

    #[tokio::test]
    async fn all_endpoint_failures_are_reported_in_file_order() -> crate::Result<()> {
        let first = TcpSocket::new_v4()?;
        first.bind("127.0.0.1:0".parse()?)?;
        let second = TcpSocket::new_v4()?;
        second.bind("127.0.0.1:0".parse()?)?;
        let endpoints = vec![format!("https://{}", first.local_addr()?), format!("https://{}", second.local_addr()?)];
        let pairing = File {
            version: 1,
            endpoints: endpoints.clone(),
            backend_identifier: "intended-host".into(),
            label: "Operator".into(),
            ca_certificate_pem: String::new(),
            access_key: "test-key".into(),
        };

        let error = pairing.channel(&Settings::default()).await.unwrap_err();
        let message = error.to_string();
        assert!(message.find(&format!("{}:", endpoints[0])).unwrap() < message.find(&format!("{}:", endpoints[1])).unwrap());
        assert!(!message.contains(&pairing.access_key));
        Ok(())
    }
}
