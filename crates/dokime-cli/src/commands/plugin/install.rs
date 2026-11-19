/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Install a Bundle Through the Host Plugin Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

// Local Imports
use crate::{Result, output, settings::Settings};
use dokime::FileMetadata;
use dokime::client::PluginsCreateOptions;
use dokime::client::plugin_placement::Kind;
use dokime::client::{
    FilesUploadsAppendOptions, FilesUploadsCreateOptions, FilesUploadsDeleteOptions, FilesUploadsPublishOptions, PluginManaged,
    PluginPlacement, PluginValue,
};

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

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: Settings) -> Result<()> {
    attach(&args.bundle, endpoint, &settings, Some(args.json)).await
}

pub(crate) async fn attach(bundle: &Path, endpoint: Option<String>, settings: &Settings, output: Option<bool>) -> Result<()> {
    let manifest = super::bundle::inspect(bundle)?;
    let name = manifest["identity"]["name"].as_str().ok_or("Install Plugin: Missing Bundle Identity")?;
    let client = settings.sdk(endpoint).await?;
    let mut file = File::open(bundle).await?;
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
    let work: Result<_> = async {
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
    let entry = value.entry.take().ok_or("Install Plugin: Missing Registered Entry")?;
    value.entry = Some(super::await_ready(&client, entry, settings, "Install Plugin").await?);
    if let Some(json) = output {
        output::print(&value, json)?;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
