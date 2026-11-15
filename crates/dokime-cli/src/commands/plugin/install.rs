/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Upload a Bundle Through the Host Plugin Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

// Local Imports
use dokime::FileMetadata;
use dokime::client::PluginsCreateOptions;
use dokime::client::plugin_placement::Kind;
use dokime::client::{
    FilesUploadsAppendOptions, FilesUploadsCreateOptions, FilesUploadsDeleteOptions, FilesUploadsPublishOptions, PluginManaged,
    PluginPlacement, PluginValue,
};
use dokime::client::{PluginState, PluginsGetOptions, PluginsWatchOptions};
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    bundle: PathBuf,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let manifest = super::bundle::inspect(&args.bundle)?;
    let name = manifest["identity"]["name"].as_str().ok_or("Install Plugin: Missing Bundle Identity")?;
    let client = settings.sdk(endpoint).await?;
    let mut file = tokio::fs::File::open(&args.bundle).await?;
    let length = file.metadata().await?.len();
    let identity = settings.request_id(None)?;
    let metadata = FileMetadata {
        expected_length: Some(length),
        ..Default::default()
    };
    let created = client
        .files
        .uploads
        .create_with(
            metadata,
            FilesUploadsCreateOptions {
                request_id: Some(format!("{identity}/create")),
                context: settings.context()?,
            },
        )
        .await?;
    let mut upload = created.upload.ok_or("Install Plugin: Missing Upload")?;
    let work: crate::Result<_> = async {
        let maximum = created
            .chunk_bytes
            .min(created.message_bytes.saturating_sub(1024))
            .min(settings.message_bytes.get().saturating_sub(1024) as u64);
        let maximum = usize::try_from(maximum).ok().filter(|size| *size > 0).ok_or("Install Plugin: Invalid Transfer Bounds")?;
        let mut buffer = vec![0; maximum];
        let mut digest = Sha256::new();
        let mut offset = 0;
        loop {
            let size = file.read(&mut buffer).await?;
            if size == 0 {
                break;
            }
            let reply = client
                .files
                .uploads
                .append_with(
                    upload.clone(),
                    offset,
                    buffer[..size].to_vec(),
                    FilesUploadsAppendOptions {
                        request_id: Some(format!("{identity}/append/{offset}")),
                        context: settings.context()?,
                    },
                )
                .await?;
            offset += size as u64;
            upload = reply.upload.ok_or("Install Plugin: Missing Accepted Upload")?;
            if upload.accepted_length != offset {
                return Err("Install Plugin: Accepted Offset Changed".into());
            }
            digest.update(&buffer[..size]);
        }
        if offset != length {
            return Err("Install Plugin: Bundle Length Changed During Upload".into());
        }
        let digest = digest.finalize().to_vec();
        let published = client
            .files
            .uploads
            .publish_with(
                upload.clone(),
                length,
                digest.clone(),
                FilesUploadsPublishOptions {
                    request_id: Some(format!("{identity}/publish")),
                    context: settings.context()?,
                },
            )
            .await?;
        let content = published.upload.ok_or("Install Plugin: Missing Published Content")?;
        if content.length != length || content.sha256 != digest {
            return Err("Install Plugin: Published Content Identity Changed".into());
        }
        Ok(content)
    }
    .await;
    if work.is_err() {
        client
            .files
            .uploads
            .delete_with(
                upload,
                FilesUploadsDeleteOptions {
                    request_id: Some(format!("{identity}/abandon")),
                    context: settings.context()?,
                },
            )
            .await
            .map_err(|cause| format!("Install Plugin: {work:?}; Abandon Upload: {cause}"))?;
    }
    let content = work?;
    let sha256 = content.sha256.clone();
    let placement = PluginPlacement {
        kind: Some(Kind::Managed(PluginManaged { content: Some(content), sha256 })),
    };
    let mut value = client
        .plugins
        .create_with(
            name,
            placement,
            PluginValue { enabled: true },
            PluginsCreateOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    let mut entry = value.entry.take().ok_or("Install Plugin: Missing Registered Entry")?;
    let mut watch = client
        .plugins
        .watch_with(PluginsWatchOptions {
            name: Some(entry.name.clone()),
            context: settings.context()?,
            ..Default::default()
        })
        .await?;
    while entry.state != PluginState::Ready as i32 {
        if matches!(PluginState::try_from(entry.state), Ok(PluginState::Unavailable | PluginState::Stopped)) {
            return Err(Status::failed_precondition(format!(
                "Install Plugin {}: {}",
                entry.name,
                entry.failure.map_or_else(|| "Activation Stopped".into(), |failure| failure.message)
            ))
            .into());
        }
        watch.next().await?.ok_or("Install Plugin: Plugin Watch Closed Before Readiness")?;
        entry = client
            .plugins
            .get_with(
                entry.name,
                PluginsGetOptions {
                    context: settings.context()?,
                    ..Default::default()
                },
            )
            .await?
            .entry
            .ok_or("Install Plugin: Plugin Entry Was Removed")?;
    }
    drop(watch);
    value.entry = Some(entry);
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
