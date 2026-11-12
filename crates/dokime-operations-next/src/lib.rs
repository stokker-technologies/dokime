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
pub mod log;
pub mod progress;
pub mod prompt;
pub mod record;
pub mod records;
mod resources;
pub mod sleep;
pub mod statistics;

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
                log::declaration()?,
                progress::declaration()?,
                record::declaration()?,
                records::declaration()?,
                statistics::declaration()?,
                prompt::declaration()?,
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
    let payload = request.payload.as_slice();
    match request.operation.as_str() {
        log::ID => Ok(log::handle(resources, v1::WriteLog::decode(payload)?).await?.encode_to_vec()),
        progress::ID => Ok(progress::handle(resources, v1::ReportProgress::decode(payload)?).await?.encode_to_vec()),
        record::ID => Ok(record::handle(resources, v1::PublishRecord::decode(payload)?).await?.encode_to_vec()),
        records::ID => Ok(records::handle(resources, v1::RecordsRequest::decode(payload)?).await?.encode_to_vec()),
        statistics::ID => Ok(statistics::handle(resources, v1::StatisticsRequest::decode(payload)?).await?.encode_to_vec()),
        prompt::ID => Ok(prompt::handle(resources, v1::OperatorQuestion::decode(payload)?).await?.encode_to_vec()),
        sleep::ID => Ok(sleep::handle(resources, h::Sleep::decode(payload)?).await?.encode_to_vec()),
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
    })
}

/* === Tests === */
