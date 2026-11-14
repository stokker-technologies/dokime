/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host Operations

/* === Modules === */

mod error;
pub mod host;
pub mod plugin;
mod resources;

/* === Re-Exports === */

pub use error::Error;
pub use resources::{Clock, Prompts, Resources};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;
use prost_reflect::ReflectMessage;

// Local Imports
use dokime_protocol::host::BackendEnvironmentRequest;
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::ReadSettings;
use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::TraceabilityGetRequest;
use dokime_protocol::host::evidence::TraceabilitySetRequest;
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_protocol::host::executions::ProgressSetRequest;
use harmos_protocol::host::TimeGetRequest;
use harmos_protocol::host::TimeSleepRequest;
use prost_types::FileDescriptorSet;
use std::sync::OnceLock;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Check the trusted caller audience independently of visibility and grants.
pub fn authorize(action: &str, caller: h::Callers) -> Result<()> {
    let (service, method) = action.split_once('/').ok_or_else(|| Error::Invalid(format!("{action}: Supply a Declared Host Operation")))?;
    let service = dokime_protocol::descriptor_pool()
        .get_service_by_name(service)
        .ok_or_else(|| Error::Invalid(format!("{action}: Select a Declared Host Service")))?;
    let method = service
        .methods()
        .find(|candidate| candidate.name() == method)
        .ok_or_else(|| Error::Invalid(format!("{action}: Select a Declared Host Method")))?;
    let declared = harmos_protocol::schema::method_callers(&method).map_err(|source| Error::Invalid(format!("{action}: {source}")))?;
    let allowed = matches!(
        (declared, caller),
        (h::Callers::All, h::Callers::Clients | h::Callers::Plugins)
            | (h::Callers::Clients, h::Callers::Clients)
            | (h::Callers::Plugins, h::Callers::Plugins)
    );
    if !allowed {
        return Err(Error::Permission(format!("{action}: This Caller Is Not Admitted")));
    }
    Ok(())
}

pub fn declarations() -> Result<&'static [h::OperationContract]> {
    static DECLARATIONS: OnceLock<std::result::Result<Vec<h::OperationContract>, String>> = OnceLock::new();
    let declarations = DECLARATIONS.get_or_init(|| {
        let declarations = || -> Result<Vec<h::OperationContract>> {
            let mut declarations = vec![
                content_artifacts_create::declaration()?,
                host::backend::environment::declaration()?,
                time::declaration()?,
                host::evidence::logs::append::declaration()?,
                progress_get::declaration()?,
                host::executions::progress::set::declaration()?,
                traceability_get::declaration()?,
                host::evidence::traceability::set::declaration()?,
                host::evidence::records::append::declaration()?,
                records::declaration()?,
                host::evidence::statistics::declaration()?,
                host::prompts::create::declaration()?,
                secret::declaration()?,
                settings::declaration()?,
                sleep::declaration()?,
            ];
            declarations.extend(files::declarations()?);
            Ok(declarations)
        };
        declarations().map_err(|error| error.to_string())
    });
    declarations.as_deref().map_err(|message| Error::Invalid(message.clone()))
}

pub async fn dispatch(resources: &Resources, request: &harmos_protocol::plugin::ChildRequest) -> Result<Vec<u8>> {
    authorize(&request.operation, h::Callers::Plugins)?;
    let declaration = declarations()?
        .iter()
        .find(|operation| operation.id == request.operation)
        .ok_or_else(|| Error::Invalid(format!("Connect a Provider for Operation {}", request.operation)))?;
    if declaration.fingerprint != request.fingerprint {
        return Err(Error::Invalid(format!("Rebuild the Plugin Against the Host Contract for {}", request.operation)));
    }
    resources.check_action_scope(&resources.execution, &request.operation)?;
    if request.operation.starts_with("dokime.host.v1.Files/") || request.operation.starts_with("dokime.host.v1.files.Uploads/") {
        return files::dispatch(resources, request).await;
    }
    let payload = request.payload.as_slice();
    match request.operation.as_str() {
        host::backend::environment::ID => {
            Ok(host::backend::environment::handle(resources, BackendEnvironmentRequest::decode(payload)?).await?.encode_to_vec())
        }
        time::ID => Ok(time::handle(resources, TimeGetRequest::decode(payload)?).await?.encode_to_vec()),
        content_artifacts_create::ID => {
            Ok(content_artifacts_create::handle(resources, ArtifactsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::logs::append::ID => {
            Ok(host::evidence::logs::append::handle(resources, LogsAppendRequest::decode(payload)?).await?.encode_to_vec())
        }
        progress_get::ID => Ok(progress_get::handle(resources, ProgressGetRequest::decode(payload)?).await?.encode_to_vec()),
        host::executions::progress::set::ID => {
            Ok(host::executions::progress::set::handle(resources, ProgressSetRequest::decode(payload)?).await?.encode_to_vec())
        }
        traceability_get::ID => Ok(traceability_get::handle(resources, TraceabilityGetRequest::decode(payload)?).await?.encode_to_vec()),
        host::evidence::traceability::set::ID => {
            Ok(host::evidence::traceability::set::handle(resources, TraceabilitySetRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::records::append::ID => {
            Ok(host::evidence::records::append::handle(resources, RecordsAppendRequest::decode(payload)?).await?.encode_to_vec())
        }
        records::ID => Ok(records::handle(resources, RecordsListRequest::decode(payload)?).await?.encode_to_vec()),
        host::evidence::statistics::ID => {
            Ok(host::evidence::statistics::handle(resources, EvidenceStatisticsRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::prompts::create::ID => {
            Ok(host::prompts::create::handle(resources, PromptsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        secret::ID => Ok(secret::handle(resources, ReadSecret::decode(payload)?).await?.encode_to_vec()),
        settings::ID => Ok(settings::handle(resources, ReadSettings::decode(payload)?).await?.encode_to_vec()),
        sleep::ID => Ok(sleep::handle(resources, TimeSleepRequest::decode(payload)?).await?.encode_to_vec()),
        _ => Err(Error::Invalid("Connect the Requested Operation Provider".into())),
    }
}

/* === Private Functions === */

pub(crate) fn declaration<Q: ReflectMessage + Default, R: ReflectMessage + Default>(
    id: &str, types: FileDescriptorSet,
) -> Result<h::OperationContract> {
    let request = Q::default().descriptor().full_name().to_owned();
    let response = R::default().descriptor().full_name().to_owned();
    let fingerprint =
        harmos_protocol::schema::fingerprint(id, &types, &request, &response).map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(h::OperationContract {
        id: id.into(),
        request,
        response,
        visibility: h::Visibility::Public as i32,
        policy: None,
        fingerprint,
        description: String::new(),
        replay_safe: false,
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::authorize;
    use crate::Error;
    use harmos_protocol::model::Callers;

    #[test]
    fn caller_admission_checks_each_direction_before_handler_dispatch() {
        for action in ["dokime.host.v1.Executions/Create", "dokime.host.v1.Secrets/Set"] {
            assert!(authorize(action, Callers::Clients).is_ok());
            assert!(matches!(authorize(action, Callers::Plugins), Err(Error::Permission(_))));
        }
        for action in ["dokime.host.v1.evidence.Records/Append", "dokime.host.v1.executions.Progress/Set"] {
            assert!(authorize(action, Callers::Plugins).is_ok());
            assert!(matches!(authorize(action, Callers::Clients), Err(Error::Permission(_))));
        }
        for action in ["dokime.host.v1.Evidence/Statistics", "dokime.host.v1.Secrets/Get", "dokime.host.v1.executions.Progress/Get"] {
            assert!(authorize(action, Callers::Clients).is_ok());
            assert!(authorize(action, Callers::Plugins).is_ok());
            assert!(matches!(authorize(action, Callers::All), Err(Error::Permission(_))));
        }
    }
}
