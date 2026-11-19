/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Connection Configuration

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashSet, fs::File, io::Read, num::NonZeroUsize, time::Duration};

// Third Party Imports
use serde::Deserialize;
use tonic::{
    codegen::http::Uri,
    metadata::MetadataMap,
    transport::{Certificate, ClientTlsConfig, Endpoint},
};

// Local Imports
use super::{CallOptions, StatusGetOptions};
use crate::{Client, ClientSettings, Error, Result, StorageTransferSettings, configuration};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ConnectionSettings {
    pub target: Option<String>,
    pub connect_timeout: Duration,
    pub certificate_pem: Option<String>,
    pub access_key: Option<String>,
    pub expected_backend_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pairing {
    version: u32,
    endpoints: Vec<String>,
    backend_identifier: String,
    label: String,
    certificate_pem: Option<String>,
    access_key: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            target: None,
            connect_timeout: Duration::from_secs(30),
            certificate_pem: None,
            access_key: None,
            expected_backend_id: None,
        }
    }
}

/* === Public Functions === */

pub async fn connect() -> Result<Client> {
    connect_with(ConnectionSettings::default()).await
}

pub async fn connect_with(settings: ConnectionSettings) -> Result<Client> {
    if settings.connect_timeout.is_zero() {
        return Err(Error::Invalid("Connect: Supply a Positive connect_timeout".into()));
    }
    let values = configuration::load()?;
    let selected = configuration::resolve("DOKIME_REMOTE", settings.target.as_deref(), &values)?;
    let mut certificate = settings.certificate_pem;
    let mut access_key = settings.access_key;
    let mut identity = settings.expected_backend_id;
    let endpoints = if selected.ends_with(".pair") {
        if certificate.is_some() || access_key.is_some() || identity.is_some() {
            return Err(Error::Invalid("Connect: Use the Pairing File's Complete Authentication Tuple Without Explicit Overrides".into()));
        }
        let mut bytes = Vec::new();
        File::open(&selected)?.take(1_048_577).read_to_end(&mut bytes)?;
        if bytes.len() > 1_048_576 {
            return Err(Error::Invalid("Read Pairing: Export a File Smaller Than 1 MiB".into()));
        }
        let document: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|error| Error::Invalid(format!("Read Pairing: Export a Version 1 Pairing: {error}")))?;
        if document.as_object().is_none_or(|value| value.len() != 6) {
            return Err(Error::Invalid("Read Pairing: Export All Six Version 1 Pairing Fields".into()));
        }
        let pairing: Pairing = serde_json::from_value(document)
            .map_err(|error| Error::Invalid(format!("Read Pairing: Export a Valid Version 1 Pairing: {error}")))?;
        let distinct = pairing.endpoints.iter().collect::<HashSet<_>>();
        if pairing.version != 1
            || !(1..=32).contains(&pairing.endpoints.len())
            || distinct.len() != pairing.endpoints.len()
            || pairing.backend_identifier.is_empty()
            || pairing.backend_identifier.len() > 128
            || pairing.backend_identifier.chars().any(|character| character.is_whitespace() || character.is_control())
            || pairing.label.trim().is_empty()
            || pairing.label.len() > 256
            || pairing.label.chars().any(char::is_control)
            || pairing
                .access_key
                .as_ref()
                .is_some_and(|key| !(32..=4096).contains(&key.len()) || !key.bytes().all(|byte| (33..=126).contains(&byte)))
        {
            return Err(Error::Invalid("Read Pairing: Export a Valid Version 1 Pairing With Distinct Endpoints and Valid Identity".into()));
        }
        certificate = pairing.certificate_pem;
        access_key = pairing.access_key;
        identity = Some(pairing.backend_identifier);
        pairing.endpoints
    } else {
        let origin = endpoint(&selected)?;
        if origin.scheme_str() != Some("http") || certificate.is_some() {
            return Err(Error::Invalid("Connect: Use a Direct HTTP Origin or a Pinned HTTPS Pairing File".into()));
        }
        vec![selected]
    };
    let mut metadata = MetadataMap::new();
    if let Some(key) = access_key {
        let value =
            format!("Bearer {key}").parse().map_err(|error| Error::Invalid(format!("Connect: Supply a Valid Access Key: {error}")))?;
        metadata.insert("authorization", value);
    }
    let context = CallOptions { metadata, ..Default::default() };
    let mut failures = Vec::new();
    for selected in endpoints {
        let attempt = async {
            let origin = endpoint(&selected)?;
            let mut endpoint = Endpoint::from_shared(selected.clone())?.connect_timeout(settings.connect_timeout);
            if origin.scheme_str() == Some("https") {
                let certificate =
                    certificate.as_deref().ok_or_else(|| Error::Invalid("Connect: HTTPS Requires a Pinned Server Certificate".into()))?;
                endpoint = endpoint.tls_config(ClientTlsConfig::new().ca_certificate(Certificate::from_pem(certificate)))?;
            } else if certificate.is_some() {
                return Err(Error::Invalid("Connect: HTTP Cannot Carry a Certificate".into()));
            }
            let channel = endpoint.connect().await?;
            let bounds = ClientSettings::for_storage(
                StorageTransferSettings::default(),
                NonZeroUsize::new(64 * 1024 * 1024).expect("Nonzero Memory Bound"),
            )?;
            let mut client = Client::from_channel(channel, bounds)?;
            client.context = context.clone();
            if let Some(expected) = &identity {
                let reply = client.status.get_with(StatusGetOptions { context: context.clone() }).await?;
                let observed = reply.status.and_then(|status| status.identity).map(|identity| identity.host_id);
                if observed.as_deref() != Some(expected) {
                    return Err(Error::Invalid("Connect: Host Identity Differs; Export a Pairing From the Intended Host".into()));
                }
            }
            Ok(client)
        };
        match tokio::time::timeout(settings.connect_timeout, attempt).await {
            Ok(Ok(client)) => return Ok(client),
            Ok(Err(error)) => failures.push(format!("{selected}: {error}")),
            Err(_) => failures.push(format!("{selected}: Connection Budget Expired")),
        }
    }
    Err(Error::Invalid(format!("Connect: {}; Check the Host Listener or Export a New Pairing", failures.join("; "))))
}

/* === Private Functions === */

fn endpoint(selected: &str) -> Result<Uri> {
    let invalid = || {
        Error::Invalid(
            "Connect: Supply an HTTP(S) Origin With an Explicit Host and Port, Without Credentials, Path, Query or Fragment".into(),
        )
    };
    if selected.chars().any(|character| character.is_whitespace() || character.is_control()) || selected.contains(['#', '@']) {
        return Err(invalid());
    }
    let origin: Uri = selected.parse().map_err(|_| invalid())?;
    let authority = origin.authority().ok_or_else(invalid)?;
    let suffix = selected.split_once("://").map(|(_, suffix)| suffix).ok_or_else(invalid)?;
    if !matches!(origin.scheme_str(), Some("http" | "https"))
        || origin.host().is_none_or(str::is_empty)
        || authority.port_u16().is_none_or(|port| port == 0)
        || suffix != authority.as_str()
    {
        return Err(invalid());
    }
    Ok(origin)
}

/* === Tests === */
