/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Display capture: the desktop portal on Linux, which writes one PNG of the
// whole desktop and needs no native display libraries; xcap elsewhere.

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

// Captures the desktop through the screenshot portal as PNG bytes.
#[cfg(all(feature = "media-native", target_os = "linux"))]
pub(super) async fn capture_png(display: Option<u32>) -> Result<Vec<u8>> {
    use super::unavailable;
    use ashpd::desktop::screenshot::Screenshot;

    if display.is_some_and(|index| index != 0) {
        return Err(unavailable("the desktop portal captures every display as one image; select display 0"));
    }
    let response = Screenshot::request()
        .interactive(false)
        .modal(false)
        .send()
        .await
        .and_then(|request| request.response())
        .map_err(|source| unavailable(format!("display capture through the portal failed: {source}")))?;
    let path = url::Url::parse(response.uri().as_str())
        .ok()
        .and_then(|uri| uri.to_file_path().ok())
        .ok_or_else(|| unavailable("display capture returned no local file"))?;

    // Take The Portal's File, Leaving No Copy In The User's Pictures
    let bytes = tokio::fs::read(&path).await.map_err(|source| unavailable(format!("display capture file is unreadable: {source}")))?;
    if let Err(source) = tokio::fs::remove_file(&path).await {
        tracing::warn!(error = %source, path = %path.display(), "portal screenshot was not removed");
    }
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(unavailable("display capture did not return a PNG image"));
    }
    Ok(bytes)
}

// Captures the indexed display, or the primary one, as PNG bytes.
#[cfg(all(feature = "media-native", not(target_os = "linux")))]
pub(super) async fn capture_png(display: Option<u32>) -> Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || capture_monitor(display)).await?
}

#[cfg(not(feature = "media-native"))]
pub(super) async fn capture_png(_display: Option<u32>) -> Result<Vec<u8>> {
    Err(super::not_compiled())
}

/* === Private Functions === */

#[cfg(all(feature = "media-native", not(target_os = "linux")))]
fn capture_monitor(display: Option<u32>) -> Result<Vec<u8>> {
    use super::unavailable;

    let monitors = xcap::Monitor::all().map_err(|source| unavailable(format!("no display is available: {source}")))?;
    let monitor = match display {
        Some(index) => monitors.get(index as usize),
        None => monitors.iter().find(|monitor| monitor.is_primary().unwrap_or(false)).or_else(|| monitors.first()),
    }
    .ok_or_else(|| unavailable("no display is available at the requested index"))?;
    let image = monitor.capture_image().map_err(|source| unavailable(format!("display capture failed: {source}")))?;

    // Encode PNG
    let mut bytes = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|source| crate::runtime::Error::Operation(format!("screenshot encoding: {source}")))?;
    Ok(bytes.into_inner())
}

/* === Tests === */
