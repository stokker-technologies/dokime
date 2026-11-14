/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

pub mod artifacts;
pub mod execution_files;
pub mod transfer;

/* === Re-Exports === */

/* === Imports === */

use crate::{
    Error, Result,
    host::{Context, storage::administration},
};
use dokime_storage::Store;
use harmos_protocol::model::FileReference;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn validate_store(store: &Store) -> Result<()> {
    if store.has_incompatible_content()? {
        return Err(Error::Precondition(
            "Open Content Storage: Select a Fresh Storage Root; This Root Uses Incompatible File Objects".into(),
        ));
    }
    Ok(())
}

pub async fn validate_root(context: &Context) -> Result<()> {
    let (entries, _) = administration::scan(context, "artifact/", None).await?;
    if !entries.is_empty() {
        return Err(Error::Precondition(
            "Open Artifact Storage: Select a Fresh Storage Root; This Root Uses Incompatible Artifact Objects".into(),
        ));
    }
    Ok(())
}

pub(crate) fn retained(context: &Context, file: &FileReference) -> Result<bool> {
    let store = context.host.store();
    let mut after = String::new();
    loop {
        let entries = store.artifact_records_page(&after, context.settings.page_items)?;
        if entries.is_empty() {
            return Ok(false);
        }
        for entry in entries {
            let artifact =
                entry.artifact.ok_or_else(|| Error::Precondition("Delete File: Restore Its Committed Artifact Metadata".into()))?;
            after = artifact.artifact_id;
            let reference = entry
                .request
                .as_ref()
                .and_then(|request| request.file.as_ref())
                .ok_or_else(|| Error::Precondition("Delete File: Restore Its Retained Artifact Reference".into()))?;
            if same_file(reference, file) {
                return Ok(true);
            }
        }
    }
}

/* === Private Functions === */

fn same_file(left: &FileReference, right: &FileReference) -> bool {
    let identity = left.metadata.as_ref().zip(right.metadata.as_ref()).is_some_and(|(left, right)| left.id == right.id);
    let history = left
        .position
        .as_ref()
        .and_then(|position| position.history.as_ref())
        .zip(right.position.as_ref().and_then(|position| position.history.as_ref()))
        .is_some_and(|(left, right)| left == right);
    identity && history
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::same_file;
    use harmos_protocol::model::{FileMetadata, FileReference, History, Position};

    #[test]
    fn retention_uses_store_identity_even_when_a_caller_changes_file_metadata() {
        let reference = FileReference {
            metadata: Some(FileMetadata {
                id: "evidence".into(),
                media_type: "text/plain".into(),
                ..Default::default()
            }),
            position: Some(Position {
                history: Some(History {
                    store: "root".into(),
                    epoch: "epoch".into(),
                }),
                sequence: 3,
            }),
            ..Default::default()
        };
        let mut altered = reference.clone();
        altered.metadata.as_mut().unwrap().media_type = "application/octet-stream".into();
        assert!(same_file(&reference, &altered));
        altered.position.as_mut().unwrap().history.as_mut().unwrap().epoch = "another-epoch".into();
        assert!(!same_file(&reference, &altered));
        assert!(!same_file(&reference, &FileReference::default()));
    }
}
