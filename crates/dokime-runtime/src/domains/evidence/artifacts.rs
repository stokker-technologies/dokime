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

// Third Party Imports

// Local Imports
use crate::{RuntimeHandle, exports::cycle::ArtifactContent, runtime::Error, runtime::Result};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthSeverity;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::HealthItemId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn add_operator_note(&self, mut request: crate::evidence_runtime::OperatorNoteRequest) -> Result<OperatorNote> {
        let runtime = self.clone();
        tokio::spawn(async move {
            let _admission = runtime.evidence_admission.lock().await;
            request.scope = Some(runtime.resolve_evidence_scope(&request.scope.clone().unwrap_or_default())?);
            let scope = request.scope.as_ref().expect("resolved note scope");
            let guards = runtime.execution_storage.admit_annotations(scope).await?;
            let note = runtime.add_operator_note_inner(request).await?;
            drop(guards);
            runtime.checkpoint_evidence_scope(&note.scope).await?;
            Ok(note)
        })
        .await
        .map_err(|error| Error::Operation(error.to_string()))?
    }

    async fn add_operator_note_inner(&self, request: crate::evidence_runtime::OperatorNoteRequest) -> Result<OperatorNote> {
        let note = OperatorNote {
            note_id: crate::model::identity::new_id::<dokime_domain::model::identity::NoteId>(),
            scope: self.state_read().canonical_note_scope(&request.scope.unwrap_or_default()).map_err(Error::Operation)?,
            author: request.author,
            message: request.message,
            created_at: crate::model::time::now_utc(),
        };
        self.commit(crate::evidence_runtime::notes::AddOperatorNote { note: note.clone() })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        Ok(note)
    }

    pub async fn read_artifact_content(&self, artifact_id: &ArtifactId, max_bytes: usize) -> Result<Option<ArtifactContent>> {
        if let Some(mut reader) = self.open_artifact_stream(artifact_id).await? {
            let record = reader.record().clone();
            let bytes = collect_artifact(&mut reader, max_bytes).await?;
            return Ok(Some(ArtifactContent { record, bytes }));
        }
        Ok(None)
    }

    pub async fn add_health_item(
        &self, scope: RecordScope, severity: HealthSeverity, message: String, detail: Option<String>,
    ) -> Result<()> {
        self.record_health_item(dokime_domain::model::execution::health::HealthItem {
            scope,
            severity,
            message,
            detail,
            health_item_id: crate::model::identity::new_id::<HealthItemId>(),
            observed_at: crate::model::time::now_utc(),
        })
        .await
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn collect_artifact(reader: &mut crate::runtime::ArtifactReader, max_bytes: usize) -> Result<Vec<u8>> {
    let too_large = || Error::Operation(format!("artifact exceeds the {max_bytes}-byte materialization limit; use streaming access"));
    if reader.record().size_bytes > max_bytes as u64 {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer).await?;
        if count == 0 {
            return Ok(bytes);
        }
        if count > max_bytes.saturating_sub(bytes.len()) {
            return Err(too_large());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

/* === Tests === */
