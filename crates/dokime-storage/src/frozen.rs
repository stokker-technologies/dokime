/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Frozen Execution Catalogs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};

// Third Party Imports
use harmos_protocol::model::Declarations;
use prost::{DecodeError, Message};
use prost_reflect::DescriptorPool;

// Local Imports
use crate::Store;

/* === Constants === */

/// Catalogs Kept Decoded; a Host Reads Few Executions at Once.
const RETAINED: usize = 8;

/* === Macros === */

/* === Type Aliases === */

/// The Latest Catalogs Read, Most Recent First, Keyed by Their Encoding.
pub(crate) type FrozenCatalogs = Mutex<VecDeque<(Vec<u8>, Arc<FrozenCatalog>)>>;

/* === Structs === */

/// An Execution's Frozen Catalog, Decoded With the Pool of Its Message Types.
pub struct FrozenCatalog {
    pub declarations: Declarations,
    /// Absent When the Catalog Declares No Types; the Pool's Error When They Do Not Resolve.
    pub pool: Option<Result<DescriptorPool, String>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FrozenCatalog {
    /// Decode a Catalog and Resolve Its Types Without Keeping Them.
    pub fn decode(catalog: &[u8]) -> Result<Self, DecodeError> {
        let declarations = Declarations::decode(catalog)?;
        let pool =
            declarations.types.clone().map(|types| DescriptorPool::from_file_descriptor_set(types).map_err(|error| error.to_string()));
        Ok(Self { declarations, pool })
    }
}

impl Store {
    /// An Execution's Frozen Catalog; a Frozen Catalog Never Changes, So the Latest Read Are Decoded Only Once.
    pub fn frozen_catalog(&self, catalog: &[u8]) -> Result<Arc<FrozenCatalog>, DecodeError> {
        let mut entries = self.frozen.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = match entries.iter().position(|(bytes, _)| bytes == catalog) {
            Some(index) => entries.remove(index).expect("a found catalog stays retained under the lock"),
            None => (catalog.to_vec(), Arc::new(FrozenCatalog::decode(catalog)?)),
        };
        let found = entry.1.clone();
        entries.push_front(entry);
        entries.truncate(RETAINED);
        Ok(found)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{RETAINED, Store};
    use harmos_protocol::model::{Declarations, StreamDeclaration};
    use prost::Message;
    use prost_types::{DescriptorProto, FileDescriptorProto, FileDescriptorSet};
    use std::sync::Arc;

    fn catalog(stream: usize) -> Vec<u8> {
        Declarations {
            types: Some(FileDescriptorSet {
                file: vec![FileDescriptorProto {
                    name: Some("fixture.proto".into()),
                    package: Some("fixture".into()),
                    message_type: vec![DescriptorProto {
                        name: Some("Row".into()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }),
            streams: vec![StreamDeclaration {
                id: format!("stream.{stream}"),
                row: "fixture.Row".into(),
                ..Default::default()
            }],
            ..Default::default()
        }
        .encode_to_vec()
    }

    #[test]
    fn reads_share_one_decoded_catalog_until_newer_catalogs_evict_it() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let (session, _cancellation) = store.read_session()?;

        // Every Read of the Same Frozen Catalog, Through the Store or a Read Session, Shares One Pool.
        let first = store.frozen_catalog(&catalog(0))?;
        assert!(Arc::ptr_eq(&first, &session.frozen_catalog(&catalog(0))?));
        assert!(first.pool.as_ref().unwrap().as_ref().unwrap().get_message_by_name("fixture.Row").is_some());

        // Only the Latest Catalogs Stay Decoded.
        for stream in 1..=RETAINED {
            store.frozen_catalog(&catalog(stream))?;
        }
        assert_eq!(store.frozen.lock().unwrap().len(), RETAINED);
        assert!(!Arc::ptr_eq(&first, &store.frozen_catalog(&catalog(0))?));
        Ok(())
    }
}
