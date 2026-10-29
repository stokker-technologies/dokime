/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{io, os::windows::ffi::OsStrExt, path::Path, ptr};

use windows_sys::Win32::{
    Foundation::{FreeLibrary, HANDLE, HMODULE},
    System::LibraryLoader::{
        BeginUpdateResourceW, EndUpdateResourceW, EnumResourceLanguagesW, EnumResourceNamesW, FindResourceExW, LOAD_LIBRARY_AS_DATAFILE,
        LOAD_LIBRARY_AS_IMAGE_RESOURCE, LoadLibraryExW, LoadResource, LockResource, SizeofResource, UpdateResourceW,
    },
};

use super::artwork::Icon;
use crate::Error;

const ICON: u16 = 3;
const GROUP_ICON: u16 = 14;
// Tauri Loads Its Default Window Icon From This Application Resource.
const APPLICATION_ICON_ID: u16 = tauri_utils::platform::WINDOWS_APP_ICON_RESOURCE_ID;

struct Module(HMODULE);

struct Update(HANDLE);

struct Resource {
    kind: u16,
    name: Name,
    language: u16,
    bytes: Vec<u8>,
}

struct Enumeration {
    resources: Vec<Resource>,
    error: Option<io::Error>,
}

enum Name {
    Id(u16),
    Text(Vec<u16>),
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0) };
    }
}

impl Drop for Update {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { EndUpdateResourceW(self.0, 1) };
        }
    }
}

impl Name {
    fn pointer(&self) -> *const u16 {
        match self {
            Self::Id(value) => identifier(*value),
            Self::Text(value) => value.as_ptr(),
        }
    }
}

pub(super) fn replace(path: &Path, icon: &Icon) -> Result<(), Error> {
    // Remove Invalidated Authenticode
    let mut bytes = std::fs::read(path)?;
    remove_signature(&mut bytes)?;
    std::fs::write(path, bytes)?;
    let old = resources(path, &[ICON, GROUP_ICON])?;

    // Replace Every Icon And Language
    let filename = wide_path(path)?;
    let handle = unsafe { BeginUpdateResourceW(filename.as_ptr(), 0) };
    if handle.is_null() {
        return Err(io::Error::last_os_error().into());
    }
    let mut update = Update(handle);

    for resource in old {
        let result =
            unsafe { UpdateResourceW(update.0, identifier(resource.kind), resource.name.pointer(), resource.language, ptr::null(), 0) };
        check(result)?;
    }
    write_resource(update.0, GROUP_ICON, APPLICATION_ICON_ID, &icon.group)?;
    for (index, frame) in icon.frames.iter().enumerate() {
        write_resource(update.0, ICON, (index + 1) as u16, frame)?;
    }
    let result = unsafe { EndUpdateResourceW(update.0, 0) };
    update.0 = ptr::null_mut();
    if result == 0 {
        return Err(io::Error::last_os_error().into());
    }
    verify(path, icon)
}

pub(super) fn verify(path: &Path, icon: &Icon) -> Result<(), Error> {
    let actual = resources(path, &[ICON, GROUP_ICON])?;
    if actual.len() != icon.frames.len() + 1 {
        let error = Error::Invalid("packaged executable has unexpected icon resources".into());
        return Err(error);
    }
    for resource in actual {
        let expected = match (&resource.name, resource.kind, resource.language) {
            (Name::Id(APPLICATION_ICON_ID), GROUP_ICON, 0) => Some(&icon.group),
            (Name::Id(id), ICON, 0) => icon.frames.get(usize::from(*id).wrapping_sub(1)),
            _ => None,
        };
        if expected == Some(&resource.bytes) {
            continue;
        }
        let error = Error::Invalid("packaged executable icon differs from organization application artwork".into());
        return Err(error);
    }
    Ok(())
}

fn check(result: i32) -> Result<(), Error> {
    if result == 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

fn identifier(value: u16) -> *const u16 {
    usize::from(value) as *const u16
}

fn wide_path(path: &Path) -> Result<Vec<u16>, Error> {
    let mut value: Vec<_> = path.as_os_str().encode_wide().collect();
    if value.contains(&0) {
        let error = Error::Invalid("executable path contains a null character".into());
        return Err(error);
    }
    value.push(0);
    Ok(value)
}

fn resources(path: &Path, kinds: &[u16]) -> Result<Vec<Resource>, Error> {
    let filename = wide_path(path)?;
    let flags = LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE;
    let handle = unsafe { LoadLibraryExW(filename.as_ptr(), ptr::null_mut(), flags) };
    if handle.is_null() {
        return Err(io::Error::last_os_error().into());
    }
    let module = Module(handle);

    let mut enumeration = Enumeration { resources: Vec::new(), error: None };
    for kind in kinds {
        enumerate_kind(module.0, *kind, &mut enumeration)?;
    }
    Ok(enumeration.resources)
}

fn enumerate_kind(module: HMODULE, kind: u16, enumeration: &mut Enumeration) -> Result<(), Error> {
    let context = (enumeration as *mut Enumeration) as isize;
    let result = unsafe { EnumResourceNamesW(module, identifier(kind), Some(enumerate_name), context) };
    if let Some(error) = enumeration.error.take() {
        return Err(error.into());
    }
    if result != 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(1813) {
        return Ok(());
    }
    Err(error.into())
}

unsafe extern "system" fn enumerate_name(module: HMODULE, kind: *const u16, name: *const u16, context: isize) -> i32 {
    let result = unsafe { EnumResourceLanguagesW(module, kind, name, Some(enumerate_language), context) };
    if result != 0 {
        return result;
    }
    let enumeration = unsafe { &mut *(context as *mut Enumeration) };
    enumeration.error.get_or_insert_with(io::Error::last_os_error);
    result
}

unsafe extern "system" fn enumerate_language(module: HMODULE, kind: *const u16, name: *const u16, language: u16, context: isize) -> i32 {
    let enumeration = unsafe { &mut *(context as *mut Enumeration) };
    let result = unsafe { read_resource(module, kind, name, language) };
    match result {
        Ok(resource) => {
            enumeration.resources.push(resource);
            1
        }
        Err(error) => {
            enumeration.error = Some(error);
            0
        }
    }
}

unsafe fn read_resource(module: HMODULE, kind: *const u16, name: *const u16, language: u16) -> Result<Resource, io::Error> {
    let resource = unsafe { FindResourceExW(module, kind, name, language) };
    let length = unsafe { SizeofResource(module, resource) };
    let loaded = unsafe { LoadResource(module, resource) };
    let data = unsafe { LockResource(loaded) };
    let invalid_resource = resource.is_null() || length == 0;
    if invalid_resource || loaded.is_null() || data.is_null() {
        return Err(io::Error::last_os_error());
    }
    let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length as usize) }.to_vec();
    let name = unsafe { resource_name(name) };
    Ok(Resource {
        kind: kind as usize as u16,
        name,
        language,
        bytes,
    })
}

unsafe fn resource_name(name: *const u16) -> Name {
    if (name as usize) <= usize::from(u16::MAX) {
        return Name::Id(name as usize as u16);
    }
    let mut text = Vec::new();
    let mut cursor = name;
    while unsafe { *cursor } != 0 {
        text.push(unsafe { *cursor });
        cursor = unsafe { cursor.add(1) };
    }
    text.push(0);
    Name::Text(text)
}

fn write_resource(handle: HANDLE, kind: u16, name: u16, bytes: &[u8]) -> Result<(), Error> {
    let length = u32::try_from(bytes.len()).map_err(|_| Error::Invalid("PE icon resource is too large".into()))?;
    let result = unsafe { UpdateResourceW(handle, identifier(kind), identifier(name), 0, bytes.as_ptr().cast(), length) };
    if result == 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

fn remove_signature(bytes: &mut Vec<u8>) -> Result<(), Error> {
    let security = security_directory(bytes)?;
    let offset = read_u32(bytes, security)? as usize;
    let length = read_u32(bytes, security + 4)? as usize;
    if offset == 0 && length == 0 {
        return Ok(());
    }
    let end = offset.checked_add(length).ok_or_else(invalid_pe)?;
    let invalid_range = end > bytes.len() || offset < security + 8;
    if offset == 0 || length < 8 || invalid_range {
        return Err(invalid_pe());
    }
    if end == bytes.len() {
        bytes.truncate(offset);
    } else {
        bytes[offset..end].fill(0);
    }
    bytes[security..security + 8].fill(0);
    Ok(())
}

fn security_directory(bytes: &[u8]) -> Result<usize, Error> {
    if bytes.get(..2) != Some(b"MZ") {
        return Err(invalid_pe());
    }
    let header = read_u32(bytes, 0x3c)? as usize;
    if bytes.get(header..header + 4) != Some(b"PE\0\0") {
        return Err(invalid_pe());
    }
    let optional = header + 24;
    let size = read_u16(bytes, header + 20)? as usize;
    let directory = match read_u16(bytes, optional)? {
        0x10b => 96,
        0x20b => 112,
        _ => return Err(invalid_pe()),
    };
    if size < directory + 40 || optional + size > bytes.len() || read_u32(bytes, optional + directory - 4)? < 5 {
        return Err(invalid_pe());
    }
    Ok(optional + directory + 32)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let field = bytes.get(offset..offset + 4).ok_or_else(invalid_pe)?;
    let value = field.try_into().map_err(|_| invalid_pe())?;
    Ok(u32::from_le_bytes(value))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, Error> {
    let field = bytes.get(offset..offset + 2).ok_or_else(invalid_pe)?;
    let value = field.try_into().map_err(|_| invalid_pe())?;
    Ok(u16::from_le_bytes(value))
}

fn invalid_pe() -> Error {
    Error::Invalid("GUI payload has an invalid PE header or Authenticode certificate directory".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, IMAGE_ICON, LR_DEFAULTSIZE, LoadImageW};

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
        let old_language =
            unsafe { UpdateResourceW(update.0, identifier(GROUP_ICON), identifier(50), 1033, b"old group".as_ptr().cast(), 9) };
        assert_ne!(old_language, 0);
        write_resource(update.0, 10, 60, b"preserved resource").unwrap();
        assert_ne!(unsafe { EndUpdateResourceW(update.0, 0) }, 0);
        std::mem::forget(update);

        let pixels = image::RgbaImage::from_pixel(64, 64, image::Rgba([20, 40, 60, 255]));
        let image = image::DynamicImage::ImageRgba8(pixels);
        let icon = super::super::artwork::icon(&image).unwrap();
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
        let different = super::super::artwork::icon(&different).unwrap();
        assert!(verify(&staged, &different).is_err());
    }

    #[test]
    fn invalidated_authenticode_is_removed() {
        let mut bytes = std::fs::read(std::env::current_exe().unwrap()).unwrap();
        remove_signature(&mut bytes).unwrap();
        let directory = security_directory(&bytes).unwrap();

        let original = bytes.clone();
        let offset = bytes.len();
        bytes.extend_from_slice(&[8, 0, 0, 0, 0, 2, 2, 0]);
        bytes[directory..directory + 4].copy_from_slice(&(offset as u32).to_le_bytes());
        bytes[directory + 4..directory + 8].copy_from_slice(&8u32.to_le_bytes());

        remove_signature(&mut bytes).unwrap();
        assert_eq!(bytes, original);
        bytes[directory + 4..directory + 8].copy_from_slice(&8u32.to_le_bytes());
        assert!(remove_signature(&mut bytes).is_err());
        assert!(remove_signature(&mut b"invalid".to_vec()).is_err());
    }
}
