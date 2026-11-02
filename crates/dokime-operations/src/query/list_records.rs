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
#[harmos::operation(resources = [crate::hosting::Host<ListRecords>], response = RecordPage, id = "dokime.query.list_records",
    policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Query", method = "ListRecords"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListRecords {
    pub scope: RecordScope,
    pub record_type: String,

    #[serde(default)]
    pub versions: Vec<u32>,

    #[serde(default)]
    pub r#where: Vec<Predicate>,
    pub between: Option<Window>,
    pub page: PageRequest,
    pub at: Option<SnapshotRef>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ListRecords {
    pub fn validate_shape(&self) -> Result<(), Box<Failure>> {
        validate_snapshot(self.at.as_ref(), Self::ID)?;
        let scope_valid = match &self.scope {
            RecordScope::Execution(id) => !id.is_empty(),
            RecordScope::Cycle(scope) => !scope.cycle_id.is_empty() && scope.execution_id.as_ref().is_none_or(|id| !id.is_empty()),
        };
        if !scope_valid {
            return Err(Failure::new(Reason::InvalidSelection, Self::ID, "Select an execution or cycle scope"));
        }
        validate_selection(&self.record_type, &self.versions, &self.r#where, self.between.as_ref(), Self::ID)?;
        validate_page(&self.page, Self::ID)
    }
}

impl Endpoint for ListRecords {
    const PATH: &'static str = "/dokime.v1.Query/ListRecords";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;
    crate::hosted!(ListRecords);
}
