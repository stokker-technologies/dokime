/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Artifacts Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::content::{StoredArtifact, artifact_key};
use crate::host::files::get;
use crate::host::{ClientSettings, storage::administration};
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::model::{ArtifactMetadata, ArtifactState};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model::ArtifactScope;

use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::content::ArtifactsCreateResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ArtifactsCreateRequest, ArtifactsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, mut request: ArtifactsCreateRequest) -> Result<ArtifactsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        request.scope = Some(scoped(context.caller.as_ref().map(|caller| caller.execution.as_str()), request.scope)?);
        if let Some(caller) = &context.caller {
            caller.check_action_scope(&caller.execution, ID)?;
        }
        let _guard = context.administration.lock().await;
        let scope = request.scope.as_ref().unwrap();
        let execution = scope.execution_id.as_deref().unwrap();
        let file = request.file.as_ref().ok_or_else(|| Error::Invalid(format!("{ID}: Select Published Content")))?;
        let metadata = file.metadata.as_ref().ok_or_else(|| Error::Invalid(format!("{ID}: Select a Named File Reference")))?;
        if metadata.scope.as_deref() != Some(execution) {
            return Err(Error::Permission(format!("{ID}: Select Published Content in Its Execution")));
        }
        if request.request_id.is_empty() {
            return Err(Error::Invalid(format!("{ID}: Supply a Request ID")));
        }
        let identifier = crate::host::views::hex(&Sha256::digest(format!("{ID}/{execution}/{}", request.request_id)));
        let key = artifact_key(&identifier);
        let snapshot = administration::read(context, &key, &[], None).await?;
        let stored = if snapshot.payload.is_empty() {
            context.host.store().require_run_open(execution)?;
            let file = get::handle(context, FilesGetRequest { file: Some(file.clone()), at: None })
                .await?
                .file
                .ok_or_else(|| Error::Precondition(format!("{ID}: Return the Published Reference")))?;
            let now_ns = context.host.now_ns();
            StoredArtifact {
                request: Some(request.clone()),
                artifact: Some(ArtifactMetadata {
                    artifact_id: identifier,
                    upload_id: metadata.id.clone(),
                    scope: request.scope.clone(),
                    filename: request.filename.clone(),
                    content_type: Some(request.content_type.clone()),
                    size_bytes: file.length,
                    sha256: crate::host::views::hex(&file.sha256),
                    state: ArtifactState::Complete as i32,
                    created_at: Some(Timestamp {
                        seconds: now_ns.div_euclid(1_000_000_000),
                        nanos: now_ns.rem_euclid(1_000_000_000) as i32,
                    }),
                    ..Default::default()
                }),
            }
        } else {
            let stored = StoredArtifact::decode(snapshot.payload.as_slice())?;
            if stored.request.as_ref() != Some(&request) {
                return Err(Error::Conflict(format!("{ID}: Use a New Request ID for Changed Content or Metadata")));
            }
            stored
        };
        let payload = if snapshot.payload.is_empty() { stored.encode_to_vec() } else { snapshot.payload };
        let (saved, receipt) = administration::replace(context, &key, Vec::new(), &request.request_id, payload).await?;
        let saved = StoredArtifact::decode(saved.payload.as_slice())?;
        let reply = ArtifactsCreateResponse {
            artifact: saved.artifact,
            receipt: Some(receipt),
        };
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

fn scoped(trusted: Option<&str>, scope: Option<ArtifactScope>) -> Result<ArtifactScope> {
    let selected = scope.as_ref().and_then(|scope| scope.execution_id.as_deref());
    let execution = crate::resources::resolve_execution(trusted, selected, ID)?.to_owned();
    let mut scope = scope.unwrap_or_default();
    if trusted.is_some() && (scope.cycle_id.is_some() || scope.case_id.is_some()) {
        return Err(Error::Permission(format!("{ID}: Select Only the Calling Execution")));
    }
    scope.execution_id = Some(execution);
    Ok(scope)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::scoped;
    use crate::Error;
    use dokime_protocol::model::ArtifactScope;

    #[test]
    fn plugin_artifacts_default_to_the_trusted_execution_and_refuse_foreign_selectors() {
        assert_eq!(scoped(Some("active"), None).unwrap().execution_id.as_deref(), Some("active"));
        let own = ArtifactScope {
            execution_id: Some("active".into()),
            ..Default::default()
        };
        assert_eq!(scoped(Some("active"), Some(own.clone())).unwrap(), own);
        let foreign = ArtifactScope {
            execution_id: Some("foreign".into()),
            ..Default::default()
        };
        assert!(matches!(scoped(Some("active"), Some(foreign)), Err(Error::Permission(_))));
        let extra = ArtifactScope {
            cycle_id: Some("another-cycle".into()),
            ..Default::default()
        };
        assert!(matches!(scoped(Some("active"), Some(extra)), Err(Error::Permission(_))));
        assert!(matches!(scoped(None, None), Err(Error::Invalid(_))));
        assert_eq!(scoped(None, Some(own.clone())).unwrap(), own);
    }
}
