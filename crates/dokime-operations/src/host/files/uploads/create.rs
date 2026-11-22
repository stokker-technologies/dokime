/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_operations::storage::{CreateFileProduction, CreateFileProductionRequest};
use harmos_runtime::Identifier;
use harmos_runtime::storage;

// Local Imports
use crate::host::Context;
use crate::host::files::transfer;
use crate::{Error, Result};
use dokime_protocol::host::files::{UploadsCreateRequest, UploadsCreateResponse};
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, mut request: UploadsCreateRequest) -> Result<UploadsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        if request.metadata.as_ref().is_some_and(|metadata| {
            ["view/", "temporary-evidence/", "prepared-report/"].iter().any(|prefix| metadata.id.starts_with(prefix))
        }) {
            return Err(Error::Invalid("Upload File: Choose an Evidence File ID Outside Reserved View, Archive and Report Names".into()));
        }
        let selected = request.metadata.as_ref().and_then(|metadata| metadata.scope.as_deref());
        let scope = scope(&context.host.identity().backend_id, context.caller.as_ref().map(|caller| caller.execution.as_str()), selected)?;
        match scope.split_once('/') {
            Some(("execution", execution)) => crate::host::evidence::require_open(context, None, Some(execution), "Upload File")?,
            Some(("cycle", cycle)) => crate::host::evidence::require_open(context, Some(cycle), None, "Upload File")?,
            _ => {}
        }
        if let Some(metadata) = request.metadata.as_mut() {
            metadata.scope = Some(scope);
        }
        derived(context, request).await
    })
    .await
}

pub(crate) async fn derived(context: &Context, request: UploadsCreateRequest) -> Result<UploadsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let mut metadata = request.metadata.ok_or_else(|| Error::Invalid(format!("{ID}: Supply File Metadata")))?;
        let owner = match metadata.scope.as_deref() {
            Some(owner) => {
                Identifier::new(owner).map_err(|cause| Error::Invalid(format!("{ID}: Select a Valid Storage Scope: {cause}")))?
            }
            None => Scope::Root(&context.host.identity().backend_id).identifier()?,
        };
        metadata.scope = Some(owner.to_string());
        let (maximum, mutation) = transfer::admission(context, ID, &request.request_id).await?;
        if metadata.id.is_empty() {
            metadata.id = mutation.key.to_string();
        }
        let metadata = storage::FileMetadata::try_from(metadata).map_err(|cause| Error::Invalid(format!("{ID}: {cause}")))?;
        let native = CreateFileProductionRequest {
            metadata: Some(metadata),
            mutation: None,
        };
        let response = transfer::write(context, ID, &request.request_id, mutation, |mutation| {
            context.storage::<CreateFileProduction>(CreateFileProductionRequest { mutation: Some(mutation), ..native.clone() })
        })
        .await?;
        let upload = response.production.map(transfer::upload);
        Ok(UploadsCreateResponse {
            upload,
            receipt: response.receipt,
            chunk_bytes: maximum as u64,
            message_bytes: context.settings.message_bytes as u64,
        })
    })
    .await
}

/* === Private Functions === */

fn scope(root: &str, execution: Option<&str>, selected: Option<&str>) -> Result<String> {
    if let Some(execution) = execution {
        let scope = Scope::Execution(execution).identifier()?.to_string();
        if selected.is_some_and(|selected| selected != scope) {
            return Err(Error::Permission(format!("{ID}: Select the Calling Execution's Storage Scope")));
        }
        return Ok(scope);
    }
    let Some(selected) = selected else {
        return Ok(Scope::Root(root).identifier()?.to_string());
    };
    let scope = match selected.split_once('/') {
        Some(("execution", key)) => Scope::Execution(key),
        Some(("cycle", key)) => Scope::Cycle(key),
        Some(("case", key)) => Scope::Case(key),
        Some(("root", key)) if key == root => Scope::Root(key),
        _ => return Err(Error::Invalid(format!("{ID}: Select an Exact Execution, Cycle, Case or Current Root Storage Scope"))),
    };
    let scope = scope.identifier().map_err(|source| Error::Invalid(format!("{ID}: {source}")))?;
    Ok(scope.to_string())
}

/* === Tests === */

crate::native::unary!(UploadsCreateRequest, UploadsCreateResponse, context, "dokime.host.v1.files", "Uploads", "Create");

#[cfg(test)]
mod tests {
    use super::scope;
    use crate::Error;

    #[test]
    fn file_scope_admission() {
        assert_eq!(scope("backend", None, None).unwrap(), "root/backend");
        assert_eq!(scope("backend", Some("same"), None).unwrap(), "execution/same");
        assert_eq!(scope("backend", Some("same"), Some("execution/same")).unwrap(), "execution/same");
        assert!(matches!(scope("backend", Some("same"), Some("cycle/same")), Err(Error::Permission(_))));
        assert!(matches!(scope("backend", Some("same"), Some("execution/other")), Err(Error::Permission(_))));
        assert!(matches!(scope("backend", Some("same"), Some("same")), Err(Error::Permission(_))));
        assert_eq!(scope("backend", None, Some("cycle/same")).unwrap(), "cycle/same");
        assert_eq!(scope("backend", None, Some("execution/same")).unwrap(), "execution/same");
        assert_eq!(scope("backend", None, Some("case/same")).unwrap(), "case/same");
        for selected in ["same", "execution/", "cycle/", "case/", "root/other"] {
            assert!(matches!(scope("backend", None, Some(selected)), Err(Error::Invalid(_))));
        }
    }
}
