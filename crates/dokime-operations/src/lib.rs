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
pub mod native;
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
use dokime_protocol::host::DocumentsAppendPartsRequest;
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::evidence::{
    LogsAppendRequest, RecordsAppendRequest, RecordsListRequest, TraceabilityGetRequest, TraceabilitySetRequest,
};
use dokime_protocol::host::executions::{ProgressGetRequest, ProgressSetRequest};
use dokime_protocol::host::files::{
    CapturesStartRequest, CapturesStopRequest, UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsGetRequest,
    UploadsPublishRequest, UploadsResumeRequest,
};
use dokime_protocol::host::{EffectsCreateRequest, EffectsGetRequest, EffectsSetRequest};
use dokime_protocol::host::{FilesCaptureRequest, FilesDeleteRequest, FilesGetRequest, FilesReadRequest};
use harmos_runtime::Failure;
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
pub fn authorize(action: &str, caller: h::Callers) -> std::result::Result<(), Failure> {
    let wrong_caller = || Failure::wrong_caller(action);
    let (service, method) = action.split_once('/').ok_or_else(wrong_caller)?;
    let service = dokime_protocol::descriptor_pool().get_service_by_name(service).ok_or_else(wrong_caller)?;
    let method = service.methods().find(|candidate| candidate.name() == method).ok_or_else(wrong_caller)?;
    let declared = harmos_protocol::schema::method_callers(&method).map_err(|_| wrong_caller())?;
    let allowed = matches!(
        (declared, caller),
        (h::Callers::All, h::Callers::Clients | h::Callers::Plugins)
            | (h::Callers::Clients, h::Callers::Clients)
            | (h::Callers::Plugins, h::Callers::Plugins)
    );
    if !allowed {
        return Err(wrong_caller());
    }
    Ok(())
}

pub fn caller_status(failure: Failure) -> tonic::Status {
    let wire = h::Failure::from(&failure);
    tonic::Status::with_details(tonic::Code::PermissionDenied, failure.to_string(), wire.encode_to_vec().into())
}

/// Complete Composed Host Declaration Inventory, Including Plugin-Only Methods.
pub fn declarations() -> Result<&'static [h::OperationContract]> {
    static DECLARATIONS: OnceLock<std::result::Result<Vec<h::OperationContract>, String>> = OnceLock::new();
    let declarations = DECLARATIONS.get_or_init(|| {
        let types = FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET).map_err(|source| source.to_string())?;
        let mut declarations = Vec::new();
        for service in dokime_protocol::descriptor_pool().services() {
            if !service.full_name().starts_with("dokime.host.v1.") && !service.full_name().starts_with("harmos.host.v1.") {
                continue;
            }
            for method in service.methods() {
                harmos_protocol::schema::method_callers(&method).map_err(|source| source.to_string())?;
                let id = format!("{}/{}", service.full_name(), method.name());
                let request = method.input().full_name().to_owned();
                let response = method.output().full_name().to_owned();
                let fingerprint =
                    harmos_protocol::schema::fingerprint(&id, &types, &request, &response).map_err(|source| source.to_string())?;
                declarations.push(h::OperationContract {
                    id,
                    request,
                    response,
                    fingerprint,
                    visibility: h::Visibility::Public as i32,
                    ..Default::default()
                });
            }
        }
        declarations.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(declarations)
    });
    declarations.as_deref().map_err(|message| Error::Invalid(message.clone()))
}

pub async fn dispatch(resources: &Resources, request: &harmos_protocol::plugin::ChildRequest) -> Result<Vec<u8>> {
    resources.context.authorize(&request.operation, h::Callers::Plugins).map_err(|source| Error::Permission(source.to_string()))?;
    let declaration = declarations()?
        .iter()
        .find(|operation| operation.id == request.operation)
        .ok_or_else(|| Error::Invalid(format!("Connect a Provider for Operation {}", request.operation)))?;
    if declaration.fingerprint != request.fingerprint {
        return Err(Error::Invalid(format!("Rebuild the Plugin Against the Host Contract for {}", request.operation)));
    }
    resources.check_action_scope(&resources.execution, &request.operation)?;
    let mut context = resources.context.as_ref().clone();
    context.caller = Some(resources.clone());
    let payload = request.payload.as_slice();
    match request.operation.as_str() {
        host::backend::environment::ID => {
            Ok(host::backend::environment::handle(resources, BackendEnvironmentRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::content::artifacts::create::ID => {
            Ok(host::content::artifacts::create::handle(&context, ArtifactsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::logs::append::ID => {
            Ok(host::evidence::logs::append::handle(resources, LogsAppendRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::executions::progress::get::ID => {
            Ok(host::executions::progress::get::handle(&context, ProgressGetRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::executions::progress::set::ID => {
            Ok(host::executions::progress::set::handle(resources, ProgressSetRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::traceability::get::ID => {
            Ok(host::evidence::traceability::get::handle(&context, TraceabilityGetRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::traceability::set::ID => {
            Ok(host::evidence::traceability::set::handle(resources, TraceabilitySetRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::records::append::ID => {
            Ok(host::evidence::records::append::handle(resources, RecordsAppendRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::records::list::ID => {
            Ok(host::evidence::records::list::handle(&context, RecordsListRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::evidence::statistics::ID => {
            Ok(host::evidence::statistics::handle(&context, EvidenceStatisticsRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::prompts::create::ID => {
            Ok(host::prompts::create::handle(resources, PromptsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::documents::append_parts::ID => {
            Ok(host::documents::append_parts::handle(resources, DocumentsAppendPartsRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::secrets::get::ID => Ok(host::secrets::get::handle(&context, SecretsGetRequest::decode(payload)?).await?.encode_to_vec()),
        host::files::get::ID => Ok(host::files::get::handle(&context, FilesGetRequest::decode(payload)?).await?.encode_to_vec()),
        host::files::read::ID => Ok(host::files::read::handle(&context, FilesReadRequest::decode(payload)?).await?.encode_to_vec()),
        host::files::delete::ID => Ok(host::files::delete::handle(&context, FilesDeleteRequest::decode(payload)?).await?.encode_to_vec()),
        host::files::uploads::create::ID => {
            Ok(host::files::uploads::create::handle(&context, UploadsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::uploads::get::ID => {
            Ok(host::files::uploads::get::handle(&context, UploadsGetRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::uploads::resume::ID => {
            Ok(host::files::uploads::resume::handle(&context, UploadsResumeRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::uploads::append::ID => {
            Ok(host::files::uploads::append::handle(&context, UploadsAppendRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::uploads::publish::ID => {
            Ok(host::files::uploads::publish::handle(&context, UploadsPublishRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::uploads::delete::ID => {
            Ok(host::files::uploads::delete::handle(&context, UploadsDeleteRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::capture::ID => {
            Ok(host::files::capture::handle(&context, FilesCaptureRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::captures::start::ID => {
            Ok(host::files::captures::start::handle(&context, CapturesStartRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::files::captures::stop::ID => {
            Ok(host::files::captures::stop::handle(&context, CapturesStopRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::effects::create::ID => {
            Ok(host::effects::create::handle(&context, EffectsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        host::effects::get::ID => Ok(host::effects::get::handle(&context, EffectsGetRequest::decode(payload)?).await?.encode_to_vec()),
        host::effects::set::ID => Ok(host::effects::set::handle(&context, EffectsSetRequest::decode(payload)?).await?.encode_to_vec()),
        "harmos.host.v1.Time/Get" | "harmos.host.v1.Time/Sleep" => {
            let operations = context.operations.with_scope(resources.execution.clone()).map_err(|source| Error::Native {
                action: "Bind Execution Time Scope",
                source,
            })?;
            operations
                .call_encoded(&request.operation, request.payload.clone())
                .await
                .map_err(|source| Error::Native { action: "Call Library Time", source })
        }
        _ => Err(Error::Invalid(format!("{}: Connect the Requested Operation Provider", request.operation))),
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
    use harmos_protocol::model::Callers;

    #[test]
    fn caller_admission_checks_each_direction_before_handler_dispatch() {
        for action in ["dokime.host.v1.Executions/Create", "dokime.host.v1.Secrets/Set"] {
            assert!(authorize(action, Callers::Clients).is_ok());
            assert!(authorize(action, Callers::Plugins).is_err());
        }
        for action in ["dokime.host.v1.evidence.Records/Append", "dokime.host.v1.executions.Progress/Set"] {
            assert!(authorize(action, Callers::Plugins).is_ok());
            assert!(authorize(action, Callers::Clients).is_err());
        }
        for action in ["dokime.host.v1.Evidence/Statistics", "dokime.host.v1.Secrets/Get", "dokime.host.v1.executions.Progress/Get"] {
            assert!(authorize(action, Callers::Clients).is_ok());
            assert!(authorize(action, Callers::Plugins).is_ok());
            assert!(authorize(action, Callers::All).is_err());
        }
    }
}
