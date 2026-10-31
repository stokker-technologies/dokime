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
#[harmos::operation(resources = [crate::hosting::Host<ListExecutions>], response = ExecutionPage, id = "dokime.query.list_executions",
    policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Query", method = "ListExecutions"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListExecutions {
    pub selection: ExecutionSelection,
    pub page: PageRequest,
    pub at: Option<SnapshotRef>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ListExecutions {
    pub fn validate_shape(&self) -> Result<(), Box<Failure>> {
        validate_snapshot(self.at.as_ref(), Self::ID)?;
        if self.selection.template_ids.iter().chain(&self.selection.cycle_ids).any(|id| id.is_empty())
            || self.selection.statuses.contains(&dokime_domain::model::execution::wire::ExecutionStatus::Unspecified)
        {
            return Err(Failure::new(Reason::InvalidSelection, Self::ID, "Use exact nonempty IDs and declared statuses"));
        }
        if let Some(window) = &self.selection.started_at {
            window.validate(Self::ID)?;
        }
        validate_page(&self.page, Self::ID)
    }
}

impl Endpoint for ListExecutions {
    const PATH: &'static str = "/dokime.v1.Query/ListExecutions";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;
    crate::hosted!(ListExecutions);
}
