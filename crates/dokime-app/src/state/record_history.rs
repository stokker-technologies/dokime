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
use std::{collections::BTreeMap, io::Write};
// Third Party Imports
use serde::{Deserialize, Serialize, ser::SerializeMap};
// Local Imports
use crate::records::RecordContract;
use dokime::core::Identifier;

/* === Constants === */
const MAX_CONTRACTS: usize = 4096;
const MAX_BYTES: usize = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct RecordHistory(BTreeMap<Identifier, BTreeMap<u32, RecordContract>>);

struct Candidate<'a> {
    history: &'a RecordHistory,
    contract: &'a RecordContract,
}

struct Versions<'a> {
    saved: Option<&'a BTreeMap<u32, RecordContract>>,
    contract: &'a RecordContract,
}

#[derive(Default)]
struct SerializedBytes {
    bytes: usize,
    exceeded: bool,
}

/* === Enums === */
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Redefinition {
    Refuse,
    Supersede,
}

/* === Traits === */

/* === Implementations === */
impl RecordHistory {
    pub(super) fn from_contracts(
        contracts: BTreeMap<Identifier, BTreeMap<u32, RecordContract>>,
    ) -> Self {
        Self(contracts)
    }

    pub(super) fn contracts(&self) -> &BTreeMap<Identifier, BTreeMap<u32, RecordContract>> {
        &self.0
    }

    pub(super) fn get(&self, kind: &Identifier, version: u32) -> Option<&RecordContract> {
        self.0.get(kind)?.get(&version)
    }

    pub(super) fn admit(&mut self, contract: &RecordContract) -> Result<(), String> {
        self.store(contract, Redefinition::Refuse)
    }

    pub(super) fn supersede(&mut self, contract: &RecordContract) -> Result<(), String> {
        self.store(contract, Redefinition::Supersede)
    }

    fn store(&mut self, contract: &RecordContract, policy: Redefinition) -> Result<(), String> {
        let exists = self
            .get(&contract.record_type, contract.schema_version)
            .is_some();
        if let Some(previous) = self.get(&contract.record_type, contract.schema_version) {
            if previous == contract {
                return Ok(());
            }
            if policy == Redefinition::Refuse {
                return Err(format!(
                    "{} version {} was previously admitted with a different definition",
                    contract.record_type, contract.schema_version
                ));
            }
        }
        if !exists && self.0.values().map(BTreeMap::len).sum::<usize>() >= MAX_CONTRACTS {
            return Err("exceeds 4096 definitions".into());
        }
        // Preserve The Serialized Quota Before Copying Or Mutating Any Declaration
        let candidate = Candidate {
            history: self,
            contract,
        };
        let mut count = SerializedBytes::default();
        if let Err(failure) = serde_json::to_writer(&mut count, &candidate) {
            return Err(if count.exceeded {
                "exceeds 4 MiB".into()
            } else {
                failure.to_string()
            });
        }
        self.0
            .entry(contract.record_type.clone())
            .or_default()
            .insert(contract.schema_version, contract.clone());
        Ok(())
    }
}

impl Serialize for Candidate<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let present = self.history.0.contains_key(&self.contract.record_type);
        let mut map =
            serializer.serialize_map(Some(self.history.0.len() + usize::from(!present)))?;
        for (kind, versions) in &self.history.0 {
            if kind == &self.contract.record_type {
                map.serialize_entry(
                    kind,
                    &Versions {
                        saved: Some(versions),
                        contract: self.contract,
                    },
                )?;
            } else {
                map.serialize_entry(kind, versions)?;
            }
        }
        if !present {
            map.serialize_entry(
                &self.contract.record_type,
                &Versions {
                    saved: None,
                    contract: self.contract,
                },
            )?;
        }
        map.end()
    }
}

impl Serialize for Versions<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let present = self
            .saved
            .is_some_and(|saved| saved.contains_key(&self.contract.schema_version));
        let count = self.saved.map_or(0, BTreeMap::len) + usize::from(!present);
        let mut map = serializer.serialize_map(Some(count))?;
        if let Some(saved) = self.saved {
            for (version, contract) in saved {
                if *version == self.contract.schema_version {
                    map.serialize_entry(version, self.contract)?;
                } else {
                    map.serialize_entry(version, contract)?;
                }
            }
        }
        if !present {
            map.serialize_entry(&self.contract.schema_version, self.contract)?;
        }
        map.end()
    }
}

impl Write for SerializedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let Some(total) = self
            .bytes
            .checked_add(bytes.len())
            .filter(|total| *total <= MAX_BYTES)
        else {
            self.exceeded = true;
            return Err(std::io::Error::other("exceeds 4 MiB"));
        };
        self.bytes = total;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
