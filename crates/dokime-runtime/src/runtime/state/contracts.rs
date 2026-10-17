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

// Local Imports
use crate::telemetry::TelemetryContract;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default)]
pub(crate) struct RuntimeContractLookup {
    telemetry: BTreeMap<Identifier, TelemetryContract>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeContractLookup {
    pub(crate) fn from_registries(telemetry: &BTreeMap<Identifier, TelemetryContract>) -> Self {
        RuntimeContractLookup { telemetry: telemetry.clone() }
    }

    pub(crate) fn telemetry_contract(&self, stream: &Identifier) -> Option<TelemetryContract> {
        self.telemetry.get(stream).cloned()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
