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
#[harmos::operation(resources = [crate::hosting::Host<ListSamples>], response = SamplePage, id = "dokime.query.list_samples",
    policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Query", method = "ListSamples"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListSamples {
    pub execution_id: String,
    pub stream: String,

    #[serde(default)]
    pub versions: Vec<u32>,

    #[serde(default)]
    pub fields: Vec<String>,

    #[serde(default)]
    pub r#where: Vec<Predicate>,
    pub between: Option<Window>,
    pub bucket: Option<BucketRequest>,
    pub page: PageRequest,
    pub at: Option<SnapshotRef>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ListSamples {
    pub fn validate_shape(&self) -> Result<(), Box<Failure>> {
        validate_snapshot(self.at.as_ref(), Self::ID)?;
        validate_samples(&self.execution_id, &self.stream, &self.versions, &self.r#where, self.between.as_ref(), Self::ID)?;
        if self.fields.iter().any(|field| field.is_empty())
            || self.fields.iter().collect::<std::collections::BTreeSet<_>>().len() != self.fields.len()
        {
            return Err(Failure::new(Reason::InvalidSelection, Self::ID, "Projection field IDs must be nonempty and unique"));
        }
        if let Some(bucket) = &self.bucket {
            bucket.validate(Self::ID)?;
        }
        validate_page(&self.page, Self::ID)
    }
}

impl Endpoint for ListSamples {
    const PATH: &'static str = "/dokime.v1.Query/ListSamples";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;
    crate::hosted!(ListSamples);
}
