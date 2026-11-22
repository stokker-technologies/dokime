/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Persisted Schema Registration

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{MutationId, SchemaInput, SetSchema, SetSchemaRequest};
use harmos_runtime::operations::OperationManager;
use harmos_runtime::Identifier;

// Local Imports
use super::io::call;
use crate::{Error, Result};
use dokime_storage::persisted::{self, bindings::{self, Binding}, records};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Register the Finite Product Graphs Once Before Admitting Native Lifecycles.
pub async fn register(operations: &OperationManager, mutation: &MutationId) -> Result<()> {
    for definition in persisted::catalog().definitions() {
        let key = definition.schema().map_err(|source| Error::Invalid(format!("Register Product Schema: {source}")))?;
        let graph = definition.graph(&Default::default()).map_err(|source| Error::Invalid(format!("Register Product Schema: {source}")))?;
        let identity = key.id.to_string();
        let request = SchemaInput { key: Some(key), graph: Some(graph) };
        set(operations, mutation, &identity, request).await?;
    }
    let declarations = [
        bindings::schema::<records::CycleEvent>()?,
        bindings::schema::<records::ExecutionEvent>()?,
        bindings::schema::<records::EvidenceEvent>()?,
        bindings::schema::<records::HealthEvent>()?,
        bindings::schema::<records::PromptEvent>()?,
        bindings::schema::<records::AnswerEvent>()?,
        bindings::schema::<records::DocumentPart>()?,
        bindings::schema::<records::ReportPublished>()?,
    ];
    for (key, graph) in declarations {
        let identity = key.id.to_string();
        let request = SchemaInput { key: Some(key), graph: Some(graph) };
        set(operations, mutation, &identity, request).await?;
    }
    Ok(())
}

/// Plugin Record/Stream IDs and Shape Versions Stay Owned by the Frozen Declaration.
pub async fn register_binding(
    operations: &OperationManager, mutation: &MutationId, binding: &Binding, bytes: NonZeroUsize,
) -> Result<()> {
    binding.validate(bytes)?;
    let schema = SchemaInput { key: Some(binding.key.clone()), graph: Some(binding.graph.clone()) };
    set(operations, mutation, binding.key.id.as_str(), schema).await
}

/* === Private Functions === */

async fn set(operations: &OperationManager, mutation: &MutationId, identity: &str, schema: SchemaInput) -> Result<()> {
    let key = Identifier::new(format!("{}/{identity}", mutation.key))
        .map_err(|source| Error::Invalid(format!("Register Product Schema: Select a Valid Mutation Identity: {source}")))?;
    let mutation = MutationId { window: mutation.window.clone(), key };
    call::<SetSchema>(operations, SetSchemaRequest {
        mutation: Some(mutation), schema: Some(schema), expected_revision: Vec::new(), create_if_absent: true,
    }).await?;
    Ok(())
}

/* === Tests === */
