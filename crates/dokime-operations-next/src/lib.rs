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

pub mod artifact;
pub mod client;
pub mod content_artifacts_create;
pub mod environment;
mod error;
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
use dokime_protocol::v1;

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
    static DECLARATIONS: std::sync::OnceLock<std::result::Result<Vec<h::OperationContract>, String>> = std::sync::OnceLock::new();
    let declarations = DECLARATIONS.get_or_init(|| {
        let declarations = || -> Result<Vec<h::OperationContract>> {
            Ok(vec![
                artifact::declaration()?,
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
            ])
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
    let payload = request.payload.as_slice();
    match request.operation.as_str() {
        environment::ID => {
            Ok(environment::handle(resources, dokime_protocol::context::BackendEnvironmentRequest::decode(payload)?).await?.encode_to_vec())
        }
        time::ID => Ok(time::handle(resources, harmos_protocol::context::TimeGetRequest::decode(payload)?).await?.encode_to_vec()),
        content_artifacts_create::ID => {
            Ok(content_artifacts_create::handle(resources, dokime_protocol::context::content::ArtifactsCreateRequest::decode(payload)?)
                .await?
                .encode_to_vec())
        }
        artifact::ID => Ok(artifact::handle(resources, v1::PublishArtifact::decode(payload)?).await?.encode_to_vec()),
        log::ID => {
            Ok(log::handle(resources, dokime_protocol::context::evidence::LogsAppendRequest::decode(payload)?).await?.encode_to_vec())
        }
        progress_get::ID => Ok(progress_get::handle(resources, dokime_protocol::context::executions::ProgressGetRequest::decode(payload)?)
            .await?
            .encode_to_vec()),
        progress_set::ID => Ok(progress_set::handle(resources, dokime_protocol::context::executions::ProgressSetRequest::decode(payload)?)
            .await?
            .encode_to_vec()),
        traceability_get::ID => {
            Ok(traceability_get::handle(resources, dokime_protocol::context::evidence::TraceabilityGetRequest::decode(payload)?)
                .await?
                .encode_to_vec())
        }
        traceability_set::ID => {
            Ok(traceability_set::handle(resources, dokime_protocol::context::evidence::TraceabilitySetRequest::decode(payload)?)
                .await?
                .encode_to_vec())
        }
        record::ID => {
            Ok(record::handle(resources, dokime_protocol::context::evidence::RecordsAppendRequest::decode(payload)?).await?.encode_to_vec())
        }
        records::ID => {
            Ok(records::handle(resources, dokime_protocol::context::evidence::RecordsListRequest::decode(payload)?).await?.encode_to_vec())
        }
        statistics::ID => {
            Ok(statistics::handle(resources, dokime_protocol::context::EvidenceStatisticsRequest::decode(payload)?).await?.encode_to_vec())
        }
        prompt::ID => {
            Ok(prompt::handle(resources, dokime_protocol::context::PromptsCreateRequest::decode(payload)?).await?.encode_to_vec())
        }
        secret::ID => Ok(secret::handle(resources, v1::ReadSecret::decode(payload)?).await?.encode_to_vec()),
        settings::ID => Ok(settings::handle(resources, v1::ReadSettings::decode(payload)?).await?.encode_to_vec()),
        sleep::ID => Ok(sleep::handle(resources, harmos_protocol::context::TimeSleepRequest::decode(payload)?).await?.encode_to_vec()),
        _ => Err(Error::Invalid("Connect the Requested Operation Provider".into())),
    }
}

/* === Private Functions === */

pub(crate) fn declaration<Q: ReflectMessage + Default, R: ReflectMessage + Default>(
    id: &str, types: prost_types::FileDescriptorSet,
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
