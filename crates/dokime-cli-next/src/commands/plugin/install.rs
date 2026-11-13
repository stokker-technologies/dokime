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
use sha2::Digest;
use tokio::io::AsyncReadExt;

// Local Imports
use dokime::v1;

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
    super::bundle::inspect(&args.bundle)?;
    let mut file = tokio::fs::File::open(&args.bundle).await?;
    let size = file.metadata().await?.len();
    let mut digest = sha2::Sha256::new();
    let chunk_bytes = settings
        .message_bytes
        .get()
        .checked_sub(1024)
        .filter(|size| *size > 0)
        .ok_or("Install Plugin: Increase message_bytes Above 1024")?;
    let mut buffer = vec![0u8; chunk_bytes.min(64 * 1024)];
    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    use tokio::io::AsyncSeekExt;
    file.rewind().await?;
    let header = v1::InstallPluginHeader {
        size_bytes: size,
        sha256: digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect(),
    };
    let (sender, receiver) = tokio::sync::mpsc::channel(settings.stream_buffer.get());
    let producer = tokio::spawn(async move {
        sender
            .send(v1::PluginsInstallInput {
                value: Some(v1::plugins_install_input::Value::Request(v1::PluginsInstallRequest {})),
            })
            .await?;
        let frame = v1::InstallPluginFrame {
            frame: Some(v1::InstallPluginFrameFrame {
                kind: Some(v1::install_plugin_frame_frame::Kind::Header(header)),
            }),
        };
        sender
            .send(v1::PluginsInstallInput {
                value: Some(v1::plugins_install_input::Value::Item(frame)),
            })
            .await?;
        let mut offset = 0;
        loop {
            let count = file.read(&mut buffer).await?;
            if count == 0 {
                break;
            }
            let chunk = v1::InstallPluginChunk { offset, data: buffer[..count].to_vec() };
            let frame = v1::InstallPluginFrame {
                frame: Some(v1::InstallPluginFrameFrame {
                    kind: Some(v1::install_plugin_frame_frame::Kind::Chunk(chunk)),
                }),
            };
            sender
                .send(v1::PluginsInstallInput {
                    value: Some(v1::plugins_install_input::Value::Item(frame)),
                })
                .await?;
            offset += count as u64;
        }
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });
    let channel = settings.channel(endpoint, false).await?;
    let response = v1::plugins_client::PluginsClient::with_interceptor(channel, settings.interceptor())
        .max_encoding_message_size(settings.message_bytes.get())
        .max_decoding_message_size(settings.message_bytes.get())
        .install(tokio_stream::wrappers::ReceiverStream::new(receiver))
        .await;
    match response {
        Ok(response) => {
            producer.await??;
            crate::output::print(&response.into_inner(), args.json)
        }
        Err(error) => {
            producer.abort();
            let _ = producer.await;
            Err(error.into())
        }
    }
}

/* === Private Functions === */

/* === Tests === */
