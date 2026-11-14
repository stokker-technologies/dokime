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

pub mod client;
pub mod content_artifacts_create;
pub mod environment;
mod error;
pub mod files;
pub mod log;
pub mod progress_get;
pub mod progress_set;
pub mod prompt;
pub mod record;
pub mod records;
mod resources;
pub mod secret;
pub mod settings;
pub mod sleep;
pub mod statistics;
pub mod time;
pub mod traceability_get;
pub mod traceability_set;

/* === Re-Exports === */

pub use error::Error;
pub use resources::{Clock, Prompts, Resources};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;
use prost_reflect::ReflectMessage;

// Local Imports
use dokime_protocol::context::BackendEnvironmentRequest;
use dokime_protocol::context::EvidenceStatisticsRequest;
use dokime_protocol::context::PromptsCreateRequest;
use dokime_protocol::context::ReadSecret;
use dokime_protocol::context::ReadSettings;
use dokime_protocol::context::content::ArtifactsCreateRequest;
use dokime_protocol::context::evidence::LogsAppendRequest;
use dokime_protocol::context::evidence::RecordsAppendRequest;
use dokime_protocol::context::evidence::RecordsListRequest;
use dokime_protocol::context::evidence::TraceabilityGetRequest;
use dokime_protocol::context::evidence::TraceabilitySetRequest;
use dokime_protocol::context::executions::ProgressGetRequest;
use dokime_protocol::context::executions::ProgressSetRequest;
use harmos_protocol::context::TimeGetRequest;
use harmos_protocol::context::TimeSleepRequest;
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

pub fn declarations() -> Result<&'static [h::OperationContract]> {
    static DECLARATIONS: OnceLock<std::result::Result<Vec<h::OperationContract>, String>> = OnceLock::new();
    let declarations = DECLARATIONS.get_or_init(|| {
        let declarations = || -> Result<Vec<h::OperationContract>> {
            let mut declarations = vec![
                content_artifacts_create::declaration()?,
                environment::declaration()?,
                time::declaration()?,
                log::declaration()?,
                progress_get::declaration()?,
                progress_set::declaration()?,
                traceability_get::declaration()?,
                traceability_set::declaration()?,
                record::declaration()?,
                records::declaration()?,
                statistics::declaration()?,
                prompt::declaration()?,
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

pub async fn dispatch(resources: &Resources, request: &h::ChildRequest) -> Result<Vec<u8>> {
    let declaration = declarations()?
        .iter()
        .find(|operation| operation.id == request.operation)
        .ok_or_else(|| Error::Invalid(format!("Connect a Provider for Operation {}", request.operation)))?;
    if declaration.fingerprint != request.fingerprint {
        return Err(Error::Invalid(format!("Rebuild the Plugin Against the Host Contract for {}", request.operation)));
    }
    resources.check_action_scope(&resources.execution, &request.operation)?;
    if request.operation.starts_with("dokime.plugin.host.v1.Files/")
        || request.operation.starts_with("dokime.plugin.host.v1.files.Uploads/")
    {
        return files::dispatch(resources, request).await;
    }
    let payload = request.payload.as_slice();
    match request.operation.as_str() {
        environment::ID => Ok(environment::handle(resources, BackendEnvironmentRequest::decode(payload)?).await?.encode_to_vec()),
        time::ID => Ok(time::handle(resources, TimeGetRequest::decode(payload)?).await?.encode_to_vec()),
        content_artifacts_create::ID => {
            Ok(content_artifacts_create::handle(resources, ArtifactsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        log::ID => Ok(log::handle(resources, LogsAppendRequest::decode(payload)?).await?.encode_to_vec()),
        progress_get::ID => Ok(progress_get::handle(resources, ProgressGetRequest::decode(payload)?).await?.encode_to_vec()),
        progress_set::ID => Ok(progress_set::handle(resources, ProgressSetRequest::decode(payload)?).await?.encode_to_vec()),
        traceability_get::ID => Ok(traceability_get::handle(resources, TraceabilityGetRequest::decode(payload)?).await?.encode_to_vec()),
        traceability_set::ID => Ok(traceability_set::handle(resources, TraceabilitySetRequest::decode(payload)?).await?.encode_to_vec()),
        record::ID => Ok(record::handle(resources, RecordsAppendRequest::decode(payload)?).await?.encode_to_vec()),
        records::ID => Ok(records::handle(resources, RecordsListRequest::decode(payload)?).await?.encode_to_vec()),
        statistics::ID => Ok(statistics::handle(resources, EvidenceStatisticsRequest::decode(payload)?).await?.encode_to_vec()),
        prompt::ID => Ok(prompt::handle(resources, PromptsCreateRequest::decode(payload)?).await?.encode_to_vec()),
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
