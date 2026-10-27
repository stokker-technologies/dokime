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

// Third Party Imports
use harmos::operations::{Direction, OperationDeclaration};
use sha2::{Digest, Sha256};

// Local Imports
use super::{Client, UploadError};
use dokime_domain::model::plugins::wire;

/* === Constants === */

const PLUGIN_CHUNK_BYTES: usize = 65_536;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    // Install one plugin bundle in the backend's advertised chunk size, which accounts for its decode retention.
    pub async fn plugins_install_bundle(
        &self, bytes: Vec<u8>, sha256: String, mut progress: impl FnMut(u64, u64) + Send + 'static,
    ) -> Result<wire::PluginInstallation, UploadError> {
        if bytes.is_empty() || bytes.len() as u64 > wire::INSTALL_ARCHIVE_MAX_BYTES {
            return Err(UploadError::Invalid("plugin archive size is outside the native limit"));
        }
        let template = wire::InstallPluginFrame {
            frame: Some(wire::InstallPluginFrameFrame::Chunk(wire::InstallPluginChunk { offset: bytes.len() as u64, data: Vec::new() })),
        };
        let chunk_bytes = chunk_bytes::<dokime_domain::operations::plugins::Install>(
            self.backend().limits.max_chunk_bytes,
            self.request_limit(),
            PLUGIN_CHUNK_BYTES,
            &template,
            &["frame", "Chunk", "field_0", "data"],
        )?;
        if chunk_bytes == 0 {
            return Err(UploadError::Invalid("negotiated frame limit cannot carry upload data"));
        }
        let total = bytes.len() as u64;
        let frames = install_frames(bytes, sha256, chunk_bytes).inspect(move |frame| {
            if let Some(wire::InstallPluginFrameFrame::Chunk(chunk)) = &frame.frame {
                progress(chunk.offset + chunk.data.len() as u64, total);
            }
        });
        Ok(self.plugins_install(tokio_stream::iter(frames)).await?)
    }
}

/* === Public Functions === */

pub fn plugin_bundle_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Private Functions === */

pub(super) fn chunk_bytes<O: OperationDeclaration>(
    advertised: Option<u32>, request_limit: usize, fallback: usize, template: &<O::Input as Direction>::Item, path: &[&str],
) -> super::Result<usize> {
    let advertised = advertised.map_or(fallback, |bytes| bytes as usize);
    harmos::grpc::input_payload_limit::<O>(template, path, advertised, request_limit)
        .map_err(|_| super::Error::Rpc(tonic::Status::invalid_argument("unable to size upload chunk for request limit")))
}

fn install_frames(bytes: Vec<u8>, sha256: String, chunk_bytes: usize) -> impl Iterator<Item = wire::InstallPluginFrame> + Send + 'static {
    let header = wire::InstallPluginFrameFrame::Header(wire::InstallPluginHeader { size_bytes: bytes.len() as u64, sha256 });
    let chunks = (0..bytes.len()).step_by(chunk_bytes).map(move |offset| {
        let end = bytes.len().min(offset + chunk_bytes);
        let data = bytes[offset..end].to_vec();
        wire::InstallPluginFrameFrame::Chunk(wire::InstallPluginChunk { offset: offset as u64, data })
    });
    std::iter::once(header).chain(chunks).map(|frame| wire::InstallPluginFrame { frame: Some(frame) })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk_template() -> wire::InstallPluginFrame {
        wire::InstallPluginFrame {
            frame: Some(wire::InstallPluginFrameFrame::Chunk(wire::InstallPluginChunk { offset: 10_000_000, data: Vec::new() })),
        }
    }

    #[test]
    fn contiguous_upload_frames() {
        let bytes = (0..=255u8).cycle().take(10 * 1024 + 7).collect::<Vec<_>>();
        let digest = plugin_bundle_digest(&bytes);
        let frames = install_frames(bytes.clone(), digest.clone(), 4096).filter_map(|frame| frame.frame).collect::<Vec<_>>();

        let wire::InstallPluginFrameFrame::Header(header) = &frames[0] else { panic!("the first frame is the header") };
        assert_eq!((header.size_bytes, header.sha256.as_str()), (bytes.len() as u64, digest.as_str()));
        let chunks = frames[1..]
            .iter()
            .map(|frame| match frame {
                wire::InstallPluginFrameFrame::Chunk(chunk) => chunk,
                wire::InstallPluginFrameFrame::Header(_) => panic!("only chunks follow the header"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            chunks.iter().map(|chunk| (chunk.offset, chunk.data.len())).collect::<Vec<_>>(),
            [(0, 4096), (4096, 4096), (8192, 2055)]
        );
        let reassembled = chunks.iter().flat_map(|chunk| chunk.data.clone()).collect::<Vec<_>>();
        assert_eq!(reassembled, bytes);
    }

    #[test]
    fn chunk_size_follows_the_advertised_limit_within_the_request_limit() {
        let fallback = 32 * 1024;
        let template = chunk_template();
        let path = ["frame", "Chunk", "field_0", "data"];
        let spacious_limit = 8 * 1024 * 1024;
        assert_eq!(
            chunk_bytes::<dokime_domain::operations::plugins::Install>(Some(64 * 1024), spacious_limit, fallback, &template, &path)
                .unwrap(),
            64 * 1024
        );
        assert_eq!(
            chunk_bytes::<dokime_domain::operations::plugins::Install>(None, spacious_limit, fallback, &template, &path).unwrap(),
            fallback
        );
        let constrained =
            chunk_bytes::<dokime_domain::operations::plugins::Install>(Some(4 * 1024 * 1024), 1024 * 1024, fallback, &template, &path)
                .unwrap();
        assert!(constrained > 0 && constrained < 1024 * 1024);
        assert_eq!(chunk_bytes::<dokime_domain::operations::plugins::Install>(Some(64 * 1024), 0, fallback, &template, &path).unwrap(), 0);
    }

    #[test]
    fn exact_chunk_multiple() {
        let frames = install_frames(vec![1; 8192], String::new(), 4096).collect::<Vec<_>>();
        assert_eq!(frames.len(), 3);
        let full = |frame: &wire::InstallPluginFrame| matches!(&frame.frame, Some(wire::InstallPluginFrameFrame::Chunk(chunk)) if chunk.data.len() == 4096);
        assert!(frames.iter().skip(1).all(full));
    }
}
