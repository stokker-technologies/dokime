/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Camera stills through nokhwa: V4L2, Media Foundation or AVFoundation.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Captures one settled frame of the indexed camera as JPEG bytes.
#[cfg(feature = "media-native")]
pub(super) fn capture_jpeg(camera: Option<u32>, warmup_frames: usize, jpeg_quality: u8) -> Result<Vec<u8>> {
    use super::unavailable;
    use nokhwa::{
        Camera,
        pixel_format::RgbFormat,
        utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    };

    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let index = CameraIndex::Index(camera.unwrap_or(0));
    let mut device = Camera::new(index, format).map_err(|source| unavailable(format!("no camera is available: {source}")))?;
    device.open_stream().map_err(|source| unavailable(format!("camera stream failed: {source}")))?;

    let mut frame = None;
    for _ in 0..=warmup_frames {
        frame = Some(device.frame().map_err(|source| unavailable(format!("camera frame failed: {source}")))?);
    }
    if let Err(source) = device.stop_stream() {
        tracing::warn!(error = %source, "camera stream did not stop cleanly");
    }
    let image = frame
        .expect("at least one camera frame")
        .decode_image::<RgbFormat>()
        .map_err(|source| unavailable(format!("camera frame decoding failed: {source}")))?;

    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, jpeg_quality)
        .encode_image(&image)
        .map_err(|source| crate::runtime::Error::Operation(format!("photo encoding: {source}")))?;
    Ok(bytes)
}

#[cfg(not(feature = "media-native"))]
pub(super) fn capture_jpeg(_camera: Option<u32>, _warmup_frames: usize, _jpeg_quality: u8) -> Result<Vec<u8>> {
    Err(super::not_compiled())
}

/* === Private Functions === */

/* === Tests === */
