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

use crate::host::ClientSettings;
use crate::host::files::get;
use dokime_protocol::host::FilesGetRequest;

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
        let file = request.file.as_ref().ok_or_else(|| Error::Invalid(format!("{ID}: Select Published Content")))?;
        get::handle(context, FilesGetRequest { file: Some(file.clone()), at: None }).await?;
        let reply = context.host.store().attach_artifact(ID, request, context.host.now_ns())?;
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

crate::native::unary!(ArtifactsCreateRequest, ArtifactsCreateResponse, context, "dokime.host.v1.content", "Artifacts", "Create");

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
