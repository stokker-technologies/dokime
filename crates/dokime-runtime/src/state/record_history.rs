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
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::records::RecordContract;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct RecordHistory(BTreeMap<Identifier, BTreeMap<u32, RecordContract>>);

/* === Enums === */

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Redefinition {
    Refuse,
    Supersede,
}

/* === Traits === */

/* === Implementations === */

impl RecordHistory {
    pub(super) fn from_contracts(contracts: BTreeMap<Identifier, BTreeMap<u32, RecordContract>>) -> Self {
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
        if let Some(previous) = self.get(&contract.record_type, contract.schema_version) {
            if previous.record_schema() == contract.record_schema() {
                return Ok(());
            }
            if policy == Redefinition::Refuse {
                return Err(format!(
                    "{} version {} was previously admitted with a different definition",
                    contract.record_type, contract.schema_version
                ));
            }
        }
        self.0.entry(contract.record_type.clone()).or_default().insert(contract.schema_version, contract.clone());
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
