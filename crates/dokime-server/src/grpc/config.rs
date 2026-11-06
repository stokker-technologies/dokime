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
use std::{net::SocketAddr, num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use zeroize::Zeroizing;

// Local Imports
use super::{
    Error,
    access::{self, Policy},
};

/* === Constants === */

const SERVER_CALLS: u64 = 32;
const SERVER_CALLS_PER_CONNECTION: u64 = 32;
const SERVER_CONNECTIONS: u64 = 64;
const SERVER_DECODE_RETAINED_BYTES: u64 = 4_194_304;
const SERVER_ENCODE_BUFFER_BYTES: u64 = 8_192;
const SERVER_INPUT_ITEM_MESSAGES: u64 = 3;
const SERVER_MAXIMUM_MESSAGE_BYTES: u64 = 16_777_216;
const SERVER_MESSAGE_BYTES: u64 = 1_048_576;
const SERVER_OPENING_TIMEOUT_MS: u64 = 5_000;
const SERVER_OUTPUT_STREAM_BYTES: u64 = 4_194_304;
const SERVER_REQUEST_TIMEOUT_MS: u64 = 86_400_000;
const SERVER_SHUTDOWN_TIMEOUT_MS: u64 = 5_000;
const SERVER_STREAM_ITEMS: u64 = 1;
pub(super) const SERVER_TLS_TIMEOUT_MS: u64 = 10_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct TlsIdentity {
    pub(super) certificate: Vec<u8>,
    pub(super) private_key: Zeroizing<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) bind_addr: SocketAddr,
    pub(crate) policy: Policy,
    pub(crate) tls: Option<TlsIdentity>,
    pub(crate) max_message_bytes: usize,
    pub(crate) trace_journal: Option<Arc<crate::trace::Journal>>,
    remote: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for TlsIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("TlsIdentity").finish_non_exhaustive()
    }
}

impl TlsIdentity {
    pub fn from_pem(certificate: Vec<u8>, private_key: Vec<u8>) -> Self {
        Self {
            certificate,
            private_key: Zeroizing::new(private_key),
        }
    }
}

impl Config {
    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    pub fn max_message_bytes(&self) -> usize {
        self.max_message_bytes
    }

    pub fn local() -> Self {
        Self {
            bind_addr: ([127, 0, 0, 1], 0).into(),
            policy: Policy::TrustedLocal,
            tls: None,
            max_message_bytes: SERVER_MESSAGE_BYTES as usize,
            trace_journal: None,
            remote: false,
        }
    }

    pub fn remote(bind_addr: SocketAddr, tls: Option<TlsIdentity>, key: Option<&str>) -> Result<Self, Error> {
        let digest = key.map(access::verifier).transpose()?;
        let mut config = Self::local();
        config.bind_addr = bind_addr;
        config.tls = tls;
        config.policy = digest.map_or(Policy::Open, Policy::Key);
        config.remote = true;
        Ok(config)
    }

    pub(super) fn managed_remote(bind_addr: SocketAddr, tls: Option<TlsIdentity>, policy: Policy, maximum: usize) -> Self {
        Self {
            bind_addr,
            policy,
            tls,
            max_message_bytes: maximum,
            remote: true,
            trace_journal: None,
        }
    }

    pub fn with_bind_addr(mut self, address: SocketAddr) -> Self {
        self.bind_addr = address;
        self
    }

    pub fn with_trace_journal(mut self, journal: Arc<crate::trace::Journal>) -> Self {
        self.trace_journal = Some(journal);
        self
    }

    pub fn with_max_message_bytes(mut self, bytes: usize) -> Self {
        self.max_message_bytes = bytes;
        self
    }

    pub(crate) fn into_listener_settings(self) -> Result<dokime::harmos::grpc::Settings, Error> {
        self.validate()?;
        let bytes = NonZeroUsize::new(self.max_message_bytes).ok_or(Error::Configuration("message limit must be positive"))?;
        let item_messages = NonZeroUsize::new(SERVER_INPUT_ITEM_MESSAGES as usize).unwrap();
        let stream = dokime::harmos::operations::StreamSettings {
            items: NonZeroUsize::new(SERVER_STREAM_ITEMS as usize).unwrap(),
            // Admitting One Decoded Input Item Charges Its Canonical Buffer And Its Decoded Value,
            // Each Nearly A Whole Message, Plus The Value's Native Layout. Two Messages Fall Short
            // By That Layout For A Full Upload Chunk, So The Single-Item Budget Holds Three.
            bytes: bytes.saturating_mul(item_messages),
        };
        let output = dokime::harmos::operations::StreamSettings {
            items: stream.items,
            bytes: NonZeroUsize::new(SERVER_OUTPUT_STREAM_BYTES as usize).ok_or(Error::Configuration("message limit must be positive"))?,
        };
        Ok(dokime::harmos::grpc::Settings {
            address: self.bind_addr,
            operations: dokime::harmos::grpc::server::Settings {
                calls: NonZeroUsize::new(SERVER_CALLS as usize).unwrap(),
                decode: dokime::harmos::grpc::DecodeSettings {
                    message_bytes: bytes,
                    retained_bytes: NonZeroUsize::new(SERVER_DECODE_RETAINED_BYTES as usize).unwrap(),
                },
                encode: dokime::harmos::grpc::EncodeSettings {
                    message_bytes: bytes,
                    buffer_bytes: NonZeroUsize::new(self.max_message_bytes.min(SERVER_ENCODE_BUFFER_BYTES as usize)).unwrap(),
                },
                input: stream,
                output,
                opening_timeout: Duration::from_millis(SERVER_OPENING_TIMEOUT_MS),
            },
            reflection: false,
            tls: None,
            connections: NonZeroUsize::new(SERVER_CONNECTIONS as usize).unwrap(),
            calls_per_connection: NonZeroUsize::new(SERVER_CALLS_PER_CONNECTION as usize).unwrap(),
            // Individual Operations Enforce Their Own Shorter Policies.
            // Do Not Cut Off A Valid Follow Stream At The Library's Ordinary Default.
            request_timeout: Duration::from_millis(SERVER_REQUEST_TIMEOUT_MS),
            shutdown_timeout: Duration::from_millis(SERVER_SHUTDOWN_TIMEOUT_MS),
        })
    }

    pub(crate) fn local_only(&self) -> bool {
        !self.remote
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        if !self.remote && !self.bind_addr.ip().is_loopback() {
            return Err(Error::Configuration("non-loopback listeners require explicit remote configuration"));
        }
        if self.remote && matches!(self.policy, Policy::TrustedLocal) {
            return Err(Error::Configuration("remote listeners cannot use native local trust"));
        }
        if self.max_message_bytes == 0 || self.max_message_bytes as u64 > SERVER_MAXIMUM_MESSAGE_BYTES {
            return Err(Error::Configuration("message limit must be positive and within the configured maximum"));
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::local()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::domain::model::{content::wire as content, plugins::wire as plugins};

    // The Largest Chunk Either Upload Frame Admits Decodes Within The Input Item Budget At Every Message Limit.
    #[test]
    fn input_budget_admits_full_chunks() {
        for message_bytes in [4_096, SERVER_MESSAGE_BYTES as usize, SERVER_MAXIMUM_MESSAGE_BYTES as usize] {
            let config = Config::local().with_max_message_bytes(message_bytes);
            let budget = config.into_listener_settings().unwrap().operations.input.bytes.get();
            let artifact = |data| content::UploadArtifactFrame {
                payload: Some(content::UploadArtifactFramePayload::Chunk(content::ArtifactChunk { offset: u64::MAX, data })),
            };
            let plugin = |data| plugins::InstallPluginFrame {
                frame: Some(plugins::InstallPluginFrameFrame::Chunk(plugins::InstallPluginChunk { offset: u64::MAX, data })),
            };
            let artifact =
                charge::<dokime::operations::content::UploadArtifact>(artifact, &["payload", "Chunk", "field_0", "data"], message_bytes);
            let plugin = charge::<dokime::operations::plugins::Install>(plugin, &["frame", "Chunk", "field_0", "data"], message_bytes);
            assert!(artifact.max(plugin) <= budget, "{message_bytes}-byte messages charge {artifact} and {plugin} of {budget}");
        }
    }

    // Charges The Decode Of The Largest Chunk A Frame Of `message_bytes` Carries.
    fn charge<O: dokime::harmos::operations::OperationDeclaration>(
        frame: impl Fn(Vec<u8>) -> <O::Input as dokime::harmos::operations::Direction>::Item, path: &[&str], message_bytes: usize,
    ) -> usize {
        let limit = dokime::harmos::grpc::input_payload_limit::<O>(&frame(Vec::new()), path, message_bytes, message_bytes).unwrap();
        let mut encoded = Vec::new();
        harmos_sdk::grpc::encode(&frame(vec![0xA5; limit]), &mut encoded).unwrap();
        let prepared = harmos_sdk::grpc::prepare::<<O::Input as dokime::harmos::operations::Direction>::Item>(
            harmos_sdk::grpc::DecodeInput::new(&encoded),
        );
        prepared.unwrap().memory()
    }
}
