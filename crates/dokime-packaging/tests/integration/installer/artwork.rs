/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Rendered Installer Sidebar Files.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Cursor;

// Third Party Imports

// Local Imports
use super::artwork_implementation::sidebar;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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
