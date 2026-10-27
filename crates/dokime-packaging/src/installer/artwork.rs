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
#[cfg(any(windows, test))]
use std::io::Cursor;

use std::path::Path;

// Third Party Imports
use image::{
    DynamicImage, ImageFormat, Rgb, RgbImage,
    codecs::ico::{IcoEncoder, IcoFrame},
    imageops::FilterType,
};

// Local Imports
use crate::Error;

/* === Constants === */

const ICON_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[cfg(any(windows, test))]
pub(super) struct Icon {
    pub group: Vec<u8>,
    pub frames: Vec<Vec<u8>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[cfg(any(windows, test))]
pub(super) fn icon(image: &DynamicImage) -> Result<Icon, Error> {
    let mut group = vec![0, 0, 1, 0, ICON_SIZES.len() as u8, 0];
    let mut frames = Vec::new();
    for (index, size) in ICON_SIZES.into_iter().enumerate() {
        let frame = image.resize_exact(size, size, FilterType::Lanczos3).to_rgba8();
        let mut encoded = Cursor::new(Vec::new());
        frame.write_to(&mut encoded, ImageFormat::Png).map_err(image_error)?;
        let bytes = encoded.into_inner();
        group.extend_from_slice(&[size as u8, size as u8, 0, 0, 1, 0, 32, 0]);
        group.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        group.extend_from_slice(&((index + 1) as u16).to_le_bytes());
        frames.push(bytes);
    }
    Ok(Icon { group, frames })
}

pub(super) fn write_ico(image: &DynamicImage, path: &Path) -> Result<(), Error> {
    let mut frames = Vec::new();
    for size in ICON_SIZES {
        let pixels = image.resize_exact(size, size, FilterType::Lanczos3).to_rgba8();
        let frame = IcoFrame::as_png(pixels.as_raw(), size, size, image::ExtendedColorType::Rgba8).map_err(image_error)?;
        frames.push(frame);
    }
    let file = std::fs::File::create(path)?;
    IcoEncoder::new(file).encode_images(&frames).map_err(image_error)
}

pub(super) fn sidebar(pack: &dokime::config::organization::ResolvedPack, path: &Path) -> Result<(), Error> {
    let color = pack.installer_sidebar_color().map_err(Error::Invalid)?;
    let bitmap = match pack.installer_sidebar_image() {
        Some((_, bytes)) => {
            let image = image::load_from_memory_with_format(bytes, ImageFormat::Png).map_err(image_error)?;
            let pixels = RgbImage::from_pixel(164, 314, Rgb(color));
            let mut background = DynamicImage::ImageRgb8(pixels);
            let image = image.resize_exact(164, 314, FilterType::Lanczos3);
            image::imageops::overlay(&mut background, &image, 0, 0);
            background.to_rgb8()
        }
        None => RgbImage::from_pixel(164, 314, Rgb(color)),
    };
    bitmap.save_with_format(path, ImageFormat::Bmp).map_err(image_error)
}

/* === Private Functions === */

fn image_error(error: image::ImageError) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_icon_sizes() {
        let pixels = RgbImage::from_pixel(32, 32, Rgb([12, 34, 56]));
        let image = DynamicImage::ImageRgb8(pixels);
        let icon = icon(&image).unwrap();
        assert_eq!(icon.frames.len(), ICON_SIZES.len());
        assert_eq!(icon.group.len(), 6 + 14 * ICON_SIZES.len());
        for (index, (bytes, size)) in icon.frames.iter().zip(ICON_SIZES).enumerate() {
            let decoded = image::load_from_memory(bytes).unwrap();
            assert_eq!(decoded.color(), image::ColorType::Rgba8);
            let decoded = decoded.to_rgb8();
            assert_eq!(decoded.dimensions(), (size, size));
            assert!(decoded.pixels().all(|pixel| pixel.0 == [12, 34, 56]));
            let entry = &icon.group[6 + index * 14..6 + (index + 1) * 14];
            assert_eq!(entry[0], size as u8);
            assert_eq!(u16::from_le_bytes(entry[12..14].try_into().unwrap()), (index + 1) as u16);
        }
    }

    #[test]
    fn sidebar_color_and_image_precedence() {
        let mut pack = dokime::config::organization::resolve_with_base(None, None).unwrap();
        pack.manifest.as_table_mut().unwrap().remove("installer");
        let scratch = tempfile::tempdir().unwrap();
        let path = scratch.path().join("sidebar.bmp");
        sidebar(&pack, &path).unwrap();
        let bitmap = image::open(&path).unwrap().to_rgb8();
        assert!(bitmap.pixels().all(|pixel| pixel.0 == [0x1b, 0x19, 0x15]));

        let installer = toml::from_str("sidebar_color = '#123456'").unwrap();
        pack.manifest.as_table_mut().unwrap().insert("installer".into(), installer);
        sidebar(&pack, &path).unwrap();
        let bitmap = image::open(&path).unwrap().to_rgb8();
        assert_eq!(bitmap.dimensions(), (164, 314));
        assert!(bitmap.pixels().all(|pixel| pixel.0 == [0x12, 0x34, 0x56]));

        let pixels = RgbImage::from_pixel(2, 2, Rgb([90, 80, 70]));
        let image = DynamicImage::ImageRgb8(pixels);
        let mut encoded = Cursor::new(Vec::new());
        image.write_to(&mut encoded, ImageFormat::Png).unwrap();
        pack.files.insert("artwork/sidebar.png".into(), encoded.into_inner());
        let installer = pack.manifest["installer"].as_table_mut().unwrap();
        installer.insert("sidebar_image".into(), toml::Value::String("artwork/sidebar.png".into()));
        sidebar(&pack, &path).unwrap();
        let bitmap = image::open(&path).unwrap().to_rgb8();
        assert!(bitmap.pixels().all(|pixel| pixel.0 == [90, 80, 70]));

        let pixels = image::RgbaImage::from_pixel(2, 2, image::Rgba([90, 80, 70, 0]));
        let transparent = DynamicImage::ImageRgba8(pixels);
        let mut encoded = Cursor::new(Vec::new());
        transparent.write_to(&mut encoded, ImageFormat::Png).unwrap();
        pack.files.insert("artwork/sidebar.png".into(), encoded.into_inner());
        sidebar(&pack, &path).unwrap();
        let bitmap = image::open(&path).unwrap().to_rgb8();
        assert!(bitmap.pixels().all(|pixel| pixel.0 == [0x12, 0x34, 0x56]));

        pack.files.insert("artwork/sidebar.png".into(), b"invalid".to_vec());
        assert!(sidebar(&pack, &path).is_err());
    }
}
