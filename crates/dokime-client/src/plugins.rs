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
use sha2::{Digest, Sha256};

// Local Imports
use super::{Client, UploadError};
use dokime_domain::model::plugins::wire;

/* === Constants === */

// Room for the frame, chunk and input envelope tags around one chunk's data.
const FRAME_OVERHEAD_BYTES: usize = 64;
// Chunk size when a backend advertises no chunk limit of its own.
const FALLBACK_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    // Stage one plugin bundle in the backend's advertised chunk size, which accounts for its decode retention.
    pub async fn plugins_stage_bundle(
        &self, bytes: Vec<u8>, sha256: String, mut progress: impl FnMut(u64, u64) + Send + 'static,
    ) -> Result<wire::PluginInstallation, UploadError> {
        if bytes.is_empty() || bytes.len() as u64 > wire::STAGE_ARCHIVE_MAX_BYTES {
            return Err(UploadError::Invalid("plugin archive size is outside the native limit"));
        }
        let chunk_bytes = chunk_bytes(self.backend().limits.max_chunk_bytes, self.request_limit());
        if chunk_bytes == 0 {
            return Err(UploadError::Invalid("negotiated frame limit cannot carry upload data"));
        }
        let total = bytes.len() as u64;
        let frames = stage_frames(bytes, sha256, chunk_bytes).inspect(move |frame| {
            if let Some(wire::StagePluginFrameFrame::Chunk(chunk)) = &frame.frame {
                progress(chunk.offset + chunk.data.len() as u64, total);
            }
        });
        Ok(self.plugins_stage(tokio_stream::iter(frames)).await?)
    }
}

/* === Public Functions === */

pub fn plugin_bundle_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Private Functions === */

pub(super) fn chunk_bytes(advertised: Option<u32>, request_limit: usize) -> usize {
    let advertised = advertised.map_or(FALLBACK_CHUNK_BYTES, |bytes| bytes as usize);
    advertised.min(request_limit.saturating_sub(FRAME_OVERHEAD_BYTES))
}

fn stage_frames(bytes: Vec<u8>, sha256: String, chunk_bytes: usize) -> impl Iterator<Item = wire::StagePluginFrame> + Send + 'static {
    let header = wire::StagePluginFrameFrame::Header(wire::StagePluginHeader { size_bytes: bytes.len() as u64, sha256 });
    let chunks = (0..bytes.len()).step_by(chunk_bytes).map(move |offset| {
        let end = bytes.len().min(offset + chunk_bytes);
        let data = bytes[offset..end].to_vec();
        wire::StagePluginFrameFrame::Chunk(wire::StagePluginChunk { offset: offset as u64, data })
    });
    std::iter::once(header).chain(chunks).map(|frame| wire::StagePluginFrame { frame: Some(frame) })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_frames_lead_with_the_header_and_cover_the_bundle_contiguously() {
        let bytes = (0..=255u8).cycle().take(10 * 1024 + 7).collect::<Vec<_>>();
        let digest = plugin_bundle_digest(&bytes);
        let frames = stage_frames(bytes.clone(), digest.clone(), 4096).filter_map(|frame| frame.frame).collect::<Vec<_>>();

        let wire::StagePluginFrameFrame::Header(header) = &frames[0] else { panic!("the first frame is the header") };
        assert_eq!((header.size_bytes, header.sha256.as_str()), (bytes.len() as u64, digest.as_str()));
        let chunks = frames[1..]
            .iter()
            .map(|frame| match frame {
                wire::StagePluginFrameFrame::Chunk(chunk) => chunk,
                wire::StagePluginFrameFrame::Header(_) => panic!("only chunks follow the header"),
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
        assert_eq!(chunk_bytes(Some(64 * 1024), 1024 * 1024), 64 * 1024);
        assert_eq!(chunk_bytes(Some(4 * 1024 * 1024), 1024 * 1024), 1024 * 1024 - FRAME_OVERHEAD_BYTES);
        assert_eq!(chunk_bytes(None, 1024 * 1024), FALLBACK_CHUNK_BYTES);
        assert_eq!(chunk_bytes(Some(64 * 1024), FRAME_OVERHEAD_BYTES), 0);
    }

    #[test]
    fn stage_frames_send_an_exact_multiple_without_an_empty_tail() {
        let frames = stage_frames(vec![1; 8192], String::new(), 4096).collect::<Vec<_>>();
        assert_eq!(frames.len(), 3);
        let full = |frame: &wire::StagePluginFrame| matches!(&frame.frame, Some(wire::StagePluginFrameFrame::Chunk(chunk)) if chunk.data.len() == 4096);
        assert!(frames.iter().skip(1).all(full));
    }
}
