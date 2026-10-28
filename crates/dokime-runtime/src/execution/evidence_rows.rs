/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Disk-Spooled Execution Evidence And Indexed Replay Receipts.

/* === Modules === */

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;

    #[test]
    fn seven_day_capture_keeps_payloads_on_disk() {
        let mut rows = EvidenceRows::<(u64, String)>::default();
        let mut previous_bytes = 0;
        for day in 0..7_u64 {
            for minute in 0..1440_u64 {
                rows.append(&(day * 1440 + minute, "x".repeat(256))).unwrap();
            }
            assert_eq!(rows.len(), (day as usize + 1) * 1440);
            assert_eq!(rows.retained_rows(), 0);
            assert!(rows.spooled_bytes() > previous_bytes);
            previous_bytes = rows.spooled_bytes();
        }
        let retained = rows.clone();
        drop(rows);
        assert_eq!(retained.retained_rows(), 0);
        for (index, row) in retained.iter().enumerate() {
            assert_eq!(row.0, index as u64);
            assert_eq!(row.1.len(), 256);
        }
    }

    #[test]
    fn spooled_identities_are_exact_and_iteration_is_ordered() {
        let mut rows = EvidenceRows::<Vec<u8>>::default();
        let first = uuid::Uuid::from_u128(1).to_string();
        let second = uuid::Uuid::from_u128(2).to_string();
        assert!(rows.push(&first, &vec![1; 1024]).unwrap());
        assert!(!rows.push(&first, &vec![2; 1024]).unwrap());
        assert!(rows.push(&second, &vec![3; 1024]).unwrap());
        assert_eq!(rows.iter().collect::<Vec<_>>(), vec![vec![1; 1024], vec![3; 1024]]);
        assert_eq!(rows.clone().len(), 2);
    }
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
#[cfg(feature = "native")]
use std::{
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

// Third Party Imports
use serde::{Serialize, de::DeserializeOwned, ser::SerializeSeq};

// Local Imports

/* === Constants === */

const MAX_ROW_BYTES: u64 = 16 * 1024 * 1024;
// One Read Fills Many Rows: Iteration Costs One Seek And Read Per Window, Not Per Row Or Byte.
#[cfg(feature = "native")]
const READ_WINDOW_BYTES: u64 = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Payloads And Identity Receipts Live On Disk; Clones Share The Same Owner.
#[derive(Debug)]
pub struct EvidenceRows<T> {
    #[cfg(feature = "native")]
    spool: Arc<Mutex<Option<Spool>>>,

    #[cfg(not(feature = "native"))]
    rows: Vec<T>,

    marker: std::marker::PhantomData<T>,
}

// One Scratch Row Keeps Its Shared Spool Alive Without Retaining Its Payload In Memory.
#[cfg(feature = "native")]
pub(crate) struct EvidenceRow<T> {
    spool: Arc<Mutex<Option<Spool>>>,
    offset: u64,
    marker: std::marker::PhantomData<T>,
}

#[cfg(feature = "native")]
#[derive(Debug)]
struct Spool {
    data: tempfile::NamedTempFile,
    index: tempfile::TempDir,
    bytes: u64,
    count: usize,
}

pub struct Rows<'a, T> {
    owner: &'a EvidenceRows<T>,
    offset: u64,
    remaining: usize,
    #[cfg(feature = "native")]
    window: Vec<u8>,
    #[cfg(feature = "native")]
    window_start: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: Clone> Clone for EvidenceRows<T> {
    fn clone(&self) -> Self {
        Self {
            #[cfg(feature = "native")]
            spool: self.spool.clone(),
            #[cfg(not(feature = "native"))]
            rows: self.rows.clone(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for EvidenceRows<T> {
    fn default() -> Self {
        Self {
            #[cfg(feature = "native")]
            spool: Arc::new(Mutex::new(None)),
            #[cfg(not(feature = "native"))]
            rows: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<T> EvidenceRows<T> {
    pub fn len(&self) -> usize {
        #[cfg(feature = "native")]
        {
            self.spool.lock().unwrap().as_ref().map_or(0, |spool| spool.count)
        }
        #[cfg(not(feature = "native"))]
        {
            self.rows.len()
        }
    }

    #[cfg(any(test, feature = "fault-injection"))]
    pub fn retained_rows(&self) -> usize {
        #[cfg(feature = "native")]
        {
            0
        }
        #[cfg(not(feature = "native"))]
        {
            self.rows.len()
        }
    }

    #[cfg(all(feature = "native", any(test, feature = "fault-injection")))]
    pub fn spooled_bytes(&self) -> u64 {
        self.spool.lock().unwrap().as_ref().map_or(0, |spool| spool.bytes)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Rows<'_, T> {
        Rows {
            owner: self,
            offset: 0,
            remaining: self.len(),
            #[cfg(feature = "native")]
            window: Vec::new(),
            #[cfg(feature = "native")]
            window_start: 0,
        }
    }
}

impl<T: Serialize + DeserializeOwned> EvidenceRows<T> {
    #[cfg(feature = "native")]
    pub(crate) fn retain(&mut self, value: &T) -> Result<EvidenceRow<T>, String> {
        let mut owner = self.spool.lock().unwrap();
        if owner.is_none() {
            *owner = Some(Spool::new().map_err(|error| error.to_string())?);
        }
        let offset = owner.as_mut().unwrap().append(value).map_err(|error| error.to_string())?;
        Ok(EvidenceRow {
            spool: Arc::clone(&self.spool),
            offset,
            marker: std::marker::PhantomData,
        })
    }

    pub(crate) fn append(&mut self, value: &T) -> Result<(), String> {
        #[cfg(feature = "native")]
        {
            let mut owner = self.spool.lock().unwrap();
            if owner.is_none() {
                *owner = Some(Spool::new().map_err(|error| error.to_string())?);
            }
            owner.as_mut().unwrap().append(value).map(|_| ()).map_err(|error| error.to_string())
        }
        #[cfg(not(feature = "native"))]
        {
            self.rows.push(
                serde_json::from_value(serde_json::to_value(value).map_err(|error| error.to_string())?)
                    .map_err(|error| error.to_string())?,
            );
            Ok(())
        }
    }

    pub(crate) fn push(&mut self, key: &str, value: &T) -> Result<bool, String> {
        #[cfg(feature = "native")]
        {
            let mut owner = self.spool.lock().unwrap();
            if owner.is_none() {
                *owner = Some(Spool::new().map_err(|error| error.to_string())?);
            }
            owner.as_mut().unwrap().push(key, value).map_err(|error| error.to_string())
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = key;
            self.rows.push(
                serde_json::from_value(serde_json::to_value(value).map_err(|error| error.to_string())?)
                    .map_err(|error| error.to_string())?,
            );
            Ok(true)
        }
    }

    pub(crate) fn selected(&self, mut selected: impl FnMut(&T) -> bool) -> Self {
        let mut rows = Self::default();
        for row in self.iter().filter(|row| selected(row)) {
            rows.append(&row).expect("selected evidence persistence failed");
        }
        rows
    }
}

#[cfg(feature = "native")]
impl<T: DeserializeOwned> EvidenceRow<T> {
    pub(crate) fn read(&self) -> Result<T, String> {
        let mut owner = self.spool.lock().unwrap();
        let mut offset = self.offset;
        owner.as_mut().expect("retained evidence has a spool").read(&mut offset).map_err(|error| error.to_string())
    }
}

impl<T: Serialize + DeserializeOwned> Serialize for EvidenceRows<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.len()))?;
        let mut rows = self.iter();
        while let Some(row) = rows.read_next() {
            sequence.serialize_element(&row.map_err(serde::ser::Error::custom)?)?;
        }
        sequence.end()
    }
}

impl<T: Serialize + DeserializeOwned> Rows<'_, T> {
    pub(crate) fn read_next(&mut self) -> Option<Result<T, String>> {
        if self.remaining == 0 {
            return None;
        }
        #[cfg(feature = "native")]
        let row = self.read_windowed().map_err(|error| error.to_string());
        #[cfg(not(feature = "native"))]
        let row = serde_json::to_value(&self.owner.rows[self.owner.len() - self.remaining])
            .and_then(serde_json::from_value)
            .map_err(|error| error.to_string());
        self.remaining -= 1;
        Some(row)
    }
}

#[cfg(feature = "native")]
impl<T: Serialize + DeserializeOwned> Rows<'_, T> {
    fn read_windowed(&mut self) -> std::io::Result<T> {
        let header = self.windowed(self.offset, 8)?;
        let length = u64::from_le_bytes(header.try_into().map_err(std::io::Error::other)?);
        if length > MAX_ROW_BYTES {
            return Err(std::io::Error::other("invalid captured evidence row length"));
        }
        let start = self.offset.checked_add(8).ok_or_else(|| std::io::Error::other("evidence offset overflow"))?;
        let row = serde_json::from_slice(self.windowed(start, length)?).map_err(std::io::Error::other)?;
        self.offset = start.checked_add(length).ok_or_else(|| std::io::Error::other("evidence offset overflow"))?;
        Ok(row)
    }

    // The Retained Window Answers When It Covers The Range; Otherwise One Locked Read Refills It.
    fn windowed(&mut self, offset: u64, length: u64) -> std::io::Result<&[u8]> {
        let end = offset.checked_add(length).ok_or_else(|| std::io::Error::other("evidence offset overflow"))?;
        let window_end = self.window_start + self.window.len() as u64;
        if offset < self.window_start || end > window_end {
            let mut owner = self.owner.spool.lock().unwrap();
            let spool = owner.as_mut().expect("nonempty evidence has a spool");
            self.window = spool.read_window(offset, length.max(READ_WINDOW_BYTES))?;
            self.window_start = offset;
            if (self.window.len() as u64) < length {
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "captured evidence row is truncated"));
            }
        }
        let start = usize::try_from(offset - self.window_start).map_err(std::io::Error::other)?;
        let length = usize::try_from(length).map_err(std::io::Error::other)?;
        Ok(&self.window[start..start + length])
    }
}

impl<T: Serialize + DeserializeOwned> Iterator for Rows<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.read_next().map(|row| row.expect("captured evidence spool must remain readable"))
    }
}

#[cfg(feature = "native")]
impl Spool {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            data: tempfile::NamedTempFile::new()?,
            index: tempfile::tempdir()?,
            bytes: 0,
            count: 0,
        })
    }

    fn key(&self, key: &str) -> std::io::Result<std::path::PathBuf> {
        let id = uuid::Uuid::parse_str(key).map_err(std::io::Error::other)?;
        let text = id.simple().to_string();
        Ok(self.index.path().join(&text[..2]).join(&text[2..]))
    }

    fn append<T: Serialize>(&mut self, value: &T) -> std::io::Result<u64> {
        let offset = self.bytes;
        let bytes = serde_json::to_vec(value).map_err(std::io::Error::other)?;
        if bytes.len() as u64 > MAX_ROW_BYTES {
            return Err(std::io::Error::other("captured evidence row exceeds frame limit"));
        }
        let next = self
            .bytes
            .checked_add(8)
            .and_then(|offset| offset.checked_add(bytes.len() as u64))
            .ok_or_else(|| std::io::Error::other("captured evidence spool offset overflow"))?;
        let count = self.count.checked_add(1).ok_or_else(|| std::io::Error::other("captured evidence row count overflow"))?;
        let mut framed = Vec::with_capacity(8 + bytes.len());
        framed.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        framed.extend_from_slice(&bytes);
        let file = self.data.as_file_mut();
        file.seek(SeekFrom::Start(self.bytes))?;
        file.write_all(&framed)?;

        self.bytes = next;
        self.count = count;
        Ok(offset)
    }

    fn push<T: Serialize>(&mut self, key: &str, value: &T) -> std::io::Result<bool> {
        let path = self.key(key)?;
        if path.exists() {
            return Ok(false);
        }
        let offset = self.bytes;
        let previous_count = self.count;
        self.append(value)?;
        // The Host Owns Durable Evidence. This Private Scratch Index Is Published
        // Only After Its Complete Payload, And A Failed Receipt Is Retried Exactly.
        let receipt = (|| -> std::io::Result<()> {
            std::fs::create_dir_all(path.parent().unwrap())?;
            let _receipt = std::fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
            Ok(())
        })();
        if let Err(error) = receipt {
            self.bytes = offset;
            self.count = previous_count;
            let _ = std::fs::remove_file(path);
            return Err(error);
        }
        Ok(true)
    }

    // Up To `length` Bytes From `offset`, Bounded By What Was Written.
    fn read_window(&mut self, offset: u64, length: u64) -> std::io::Result<Vec<u8>> {
        let available = self.bytes.saturating_sub(offset).min(length);
        let mut window = vec![0; usize::try_from(available).map_err(std::io::Error::other)?];
        let file = self.data.as_file_mut();
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut window)?;
        Ok(window)
    }

    fn read<T: DeserializeOwned>(&mut self, offset: &mut u64) -> std::io::Result<T> {
        let file = self.data.as_file_mut();
        file.seek(SeekFrom::Start(*offset))?;
        let mut length = [0; 8];
        file.read_exact(&mut length)?;
        let length = u64::from_le_bytes(length);
        if length > MAX_ROW_BYTES {
            return Err(std::io::Error::other("invalid captured evidence row length"));
        }
        let mut payload = vec![0; usize::try_from(length).map_err(std::io::Error::other)?];
        file.read_exact(&mut payload)?;
        let row = serde_json::from_slice(&payload).map_err(std::io::Error::other)?;
        *offset = offset.checked_add(8 + length).ok_or_else(|| std::io::Error::other("evidence offset overflow"))?;
        Ok(row)
    }
}

impl<T: Serialize + DeserializeOwned + PartialEq> PartialEq for EvidenceRows<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(left, right)| left == right)
    }
}

impl<T: Serialize + DeserializeOwned> From<Vec<T>> for EvidenceRows<T> {
    fn from(values: Vec<T>) -> Self {
        let mut rows = Self::default();
        for value in values {
            rows.append(&value).expect("evidence spool write failed");
        }
        rows
    }
}

impl<'de, T: Serialize + DeserializeOwned> serde::Deserialize<'de> for EvidenceRows<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: Serialize + DeserializeOwned> serde::de::Visitor<'de> for Visitor<T> {
            type Value = EvidenceRows<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("paged evidence rows")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
                let mut rows = EvidenceRows::default();
                while let Some(row) = sequence.next_element::<T>()? {
                    rows.append(&row).map_err(serde::de::Error::custom)?;
                }
                Ok(rows)
            }
        }
        deserializer.deserialize_seq(Visitor(std::marker::PhantomData))
    }
}

impl<'a, T: Serialize + DeserializeOwned> IntoIterator for &'a EvidenceRows<T> {
    type Item = T;
    type IntoIter = Rows<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/* === Public Functions === */

pub(crate) fn encoded_bytes(value: &impl Serialize) -> usize {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.checked_add(bytes.len()).ok_or_else(|| std::io::Error::other("evidence byte count overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value).expect("captured evidence serializes");
    counter.0
}

/* === Private Functions === */

/* === Tests === */
