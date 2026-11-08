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

// Local Imports

use dokime_gui::testing::prepare_image_preview as prepare_image;
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
fn image_preview_is_bounded_and_decodable() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("preview");
    image::RgbImage::new(2400, 1600).save_with_format(&path, image::ImageFormat::Png).unwrap();
    assert!(prepare_image(&path).unwrap());
    let image = image::ImageReader::open(&path).unwrap().with_guessed_format().unwrap().decode().unwrap();
    assert_eq!((image.width(), image.height()), (960, 640));

    // Oversized Header Refuses Decode Before Allocating Its Pixel Buffer.
    let huge = image::RgbImage::new(1, 32_769);
    huge.save_with_format(&path, image::ImageFormat::Png).unwrap();
    assert!(prepare_image(&path).is_err());

    std::fs::write(&path, b"RIFF\x04\x00\x00\x00WAVE").unwrap();
    assert!(!prepare_image(&path).unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), b"RIFF\x04\x00\x00\x00WAVE");
}
