/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Read-Only Query Request

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::operations::OperationDeclaration;
use serde::{Deserialize, Serialize};

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Active Work Uses Runtime Query Configuration Within The Declared Outer Ceiling.
#[harmos::operation(resources = [crate::hosting::Host<ListCycles>], response = CyclePage, id = "dokime.query.list_cycles",
    policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Query", method = "ListCycles"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListCycles {
    pub selection: CycleSelection,
    pub page: PageRequest,
    pub at: Option<SnapshotRef>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ListCycles {
    pub fn validate_shape(&self) -> Result<(), Box<Failure>> {
        validate_snapshot(self.at.as_ref(), Self::ID)?;
        if self.selection.cycle_ids.iter().any(|id| id.is_empty()) {
            return Err(Failure::new(Reason::InvalidSelection, Self::ID, "Use exact nonempty cycle IDs"));
        }
        if let Some(window) = &self.selection.created_at {
            window.validate(Self::ID)?;
        }
        validate_page(&self.page, Self::ID)
    }
}

impl Endpoint for ListCycles {
    const PATH: &'static str = "/dokime.v1.Query/ListCycles";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;
    crate::hosted!(ListCycles);
}
