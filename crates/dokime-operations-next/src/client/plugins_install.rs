/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::{Digest, Sha256};
use tokio_stream::{Stream, StreamExt};

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.plugins.install";
pub const BUNDLE_BYTES: usize = 285212672;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

pub enum Upload {
    Request(v1::PluginsInstallRequest),
    Item(v1::InstallPluginFrame),
    Finish,
    Cancel,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PluginsInstallRequest, v1::PluginInstallation>(ID, settings.mutation_timeout)
}

pub async fn handle(
    context: &Context, mut input: impl Stream<Item = std::result::Result<Upload, tonic::Status>> + Unpin,
) -> Result<v1::PluginInstallation> {
    let deadline = tokio::time::Instant::now() + context.settings.mutation_timeout;
    let bytes = super::complete(ID, context.settings.mutation_timeout, async {
        let Some(Upload::Request(_)) = input.next().await.transpose().map_err(|_| invalid("Send One Request First"))? else {
            return Err(invalid("Send One Request First"));
        };
        let mut header = None;
        let mut bytes = Vec::new();
        while let Some(frame) = input.next().await {
            match frame.map_err(|_| invalid("Reconnect and Reconcile Installations Before Retrying the Upload"))? {
                Upload::Cancel => return Err(Error::Cancelled("Install Plugin: Upload Cancelled Before Installation".into())),
                Upload::Finish => break,
                Upload::Request(_) => return Err(invalid("Send Exactly One Request Before the Upload")),
                Upload::Item(frame) => match frame.frame.and_then(|frame| frame.kind) {
                    Some(v1::install_plugin_frame_frame::Kind::Header(value)) if header.is_none() => {
                        if value.size_bytes == 0
                            || value.size_bytes > BUNDLE_BYTES as u64
                            || value.sha256.len() != 64
                            || !value.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
                        {
                            return Err(invalid("Supply a Bounded Nonempty Archive Size and SHA-256"));
                        }
                        header = Some(value);
                    }
                    Some(v1::install_plugin_frame_frame::Kind::Chunk(chunk)) => {
                        let expected = header.as_ref().ok_or_else(|| invalid("Send the Upload Header Before Its Chunks"))?;
                        if chunk.offset != bytes.len() as u64
                            || chunk.data.is_empty()
                            || chunk.data.len() > context.settings.message_bytes / 2
                            || bytes.len().saturating_add(chunk.data.len()) > expected.size_bytes as usize
                        {
                            return Err(invalid("Send Contiguous Nonempty Chunks Within the Advertised Size and Message Budget"));
                        }
                        bytes.extend(chunk.data);
                    }
                    _ => return Err(invalid("Send One Header Followed by Contiguous Chunks")),
                },
            }
        }
        let header = header.ok_or_else(|| invalid("Send the Upload Header and Archive Bytes"))?;
        let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        if bytes.len() as u64 != header.size_bytes || digest != header.sha256.to_ascii_lowercase() {
            return Err(invalid("Retry the Upload with Its Complete Size and Matching SHA-256"));
        }
        Ok(bytes)
    })
    .await?;

    // Keep the Original Deadline and Settle Rollback Before Returning Its Failure
    let host = context.host.clone();
    let administration = context.administration.clone();
    let (cancel, mut cancelled) = tokio::sync::oneshot::channel();
    let mut task = tokio::spawn(async move {
        let _guard = tokio::select! {
            biased;
            _ = &mut cancelled => return Err(Error::Cancelled(ID.into())),
            guard = administration.lock() => guard,
        };
        host.install_plugin_bundle(bytes, cancelled).await
    });
    let cancellation = async {
        while let Some(frame) = input.next().await {
            match frame {
                Ok(Upload::Cancel) => return Ok::<(), Error>(()),
                Err(_) => break,
                _ => return Err(invalid("Send Only Cancel After Finishing the Upload")),
            }
        }
        std::future::pending::<Result<()>>().await
    };
    let failure = tokio::select! {
        biased;
        result = &mut task => return result.map_err(|_| invalid("Refresh the Installation List Before Retrying After an Uncertain Acknowledgement"))?,
        _ = tokio::time::sleep_until(deadline) => Error::Deadline(ID.into()),
        result = cancellation => result.err().unwrap_or_else(|| Error::Cancelled("Install Plugin: Cancelled by the Client".into())),
    };
    let _ = cancel.send(());
    match task.await.map_err(|_| invalid("Refresh the Installation List After Interrupted Rollback"))? {
        // A Completed Commit Wins Cancellation; Report Its Durable Result.
        Ok(reply) => Ok(reply),
        Err(Error::Cancelled(_)) => Err(failure),
        Err(error) => Err(error),
    }
}

/* === Private Functions === */

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("Install Plugin: {message}"))
}

/* === Tests === */
