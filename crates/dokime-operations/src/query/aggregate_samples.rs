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
#[harmos::operation(resources = [crate::hosting::Host<AggregateSamples>], response = Statistics, id = "dokime.query.aggregate_samples",
    policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Query", method = "AggregateSamples"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateSamples {
    pub execution_id: String,
    pub stream: String,

    #[serde(default)]
    pub versions: Vec<u32>,

    #[serde(default)]
    pub r#where: Vec<Predicate>,
    pub between: Option<Window>,

    #[serde(default)]
    pub outputs: Vec<Output>,
    pub at: Option<SnapshotRef>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl AggregateSamples {
    pub fn validate_shape(&self) -> Result<(), Box<Failure>> {
        validate_snapshot(self.at.as_ref(), Self::ID)?;
        validate_samples(&self.execution_id, &self.stream, &self.versions, &self.r#where, self.between.as_ref(), Self::ID)?;
        if self.outputs.is_empty() {
            return Err(Failure::new(Reason::InvalidSelection, Self::ID, "Select at least one aggregate output"));
        }
        let mut aliases = std::collections::BTreeSet::new();
        for output in &self.outputs {
            output.validate(Self::ID)?;
            if !aliases.insert(&output.alias) {
                return Err(Failure::new(Reason::InvalidSelection, Self::ID, "Aggregate aliases must be unique"));
            }
        }
        Ok(())
    }
}

impl Endpoint for AggregateSamples {
    const PATH: &'static str = "/dokime.v1.Query/AggregateSamples";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;
    crate::hosted!(AggregateSamples);
}
