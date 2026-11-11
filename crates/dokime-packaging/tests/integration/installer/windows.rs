/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Native Windows Resource Replacement.

/* === Modules === */

include!("../../../src/installer/windows.rs");

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, IMAGE_ICON, LR_DEFAULTSIZE, LoadImageW};

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
fn staged_icon_replaces_all_languages_and_preserves_payload() {
    let source = std::env::current_exe().unwrap();
    let original = std::fs::read(&source).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let staged = scratch.path().join("gui.exe");
    std::fs::write(&staged, &original).unwrap();

    let filename = wide_path(&staged).unwrap();
    let update = Update(unsafe { BeginUpdateResourceW(filename.as_ptr(), 0) });
    assert!(!update.0.is_null());
    write_resource(update.0, ICON, 40, b"old icon").unwrap();
    let old_language = unsafe { UpdateResourceW(update.0, identifier(GROUP_ICON), identifier(50), 1033, b"old group".as_ptr().cast(), 9) };
    assert_ne!(old_language, 0);
    write_resource(update.0, 10, 60, b"preserved resource").unwrap();
    assert_ne!(unsafe { EndUpdateResourceW(update.0, 0) }, 0);
    std::mem::forget(update);

    let pixels = image::RgbaImage::from_pixel(64, 64, image::Rgba([20, 40, 60, 255]));
    let image = image::DynamicImage::ImageRgba8(pixels);
    let icon = super::artwork_implementation::icon(&image).unwrap();
    replace(&staged, &icon).unwrap();
    verify(&staged, &icon).unwrap();

    let expected_id = tauri_utils::platform::WINDOWS_APP_ICON_RESOURCE_ID;
    assert_eq!(expected_id, 32512);
    let groups = resources(&staged, &[GROUP_ICON]).unwrap();
    assert_eq!(groups.len(), 1);
    assert!(matches!(groups[0].name, Name::Id(id) if id == expected_id));

    let flags = LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE;
    let handle = unsafe { LoadLibraryExW(filename.as_ptr(), ptr::null_mut(), flags) };
    assert!(!handle.is_null());
    let module = Module(handle);
    let loaded = unsafe { LoadImageW(module.0, identifier(expected_id), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE) };
    assert!(!loaded.is_null(), "the patched application icon must load through the native framework resource lookup");
    assert_ne!(unsafe { DestroyIcon(loaded) }, 0);
    drop(module);

    let preserved = resources(&staged, &[10]).unwrap();
    assert!(preserved.iter().any(|resource| resource.bytes == b"preserved resource"));
    assert_eq!(std::fs::read(source).unwrap(), original);
    assert_ne!(std::fs::read(&staged).unwrap(), original);
    let pixels = image::RgbImage::from_pixel(32, 32, image::Rgb([1, 2, 3]));
    let different = image::DynamicImage::ImageRgb8(pixels);
    let different = super::artwork_implementation::icon(&different).unwrap();
    assert!(verify(&staged, &different).is_err());
}
