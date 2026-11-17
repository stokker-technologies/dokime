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
use std::result;

// Third Party Imports
use harmos_protocol::model;
use harmos_runtime::Failure;
use prost_reflect::ReflectMessage;
use prost_types::FileDescriptorSet;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Check the trusted caller audience independently of visibility and grants.
pub fn authorize(action: &str, caller: model::Callers) -> result::Result<(), Failure> {
    let wrong_caller = || Failure::wrong_caller(action);
    let (service, method) = action.split_once('/').ok_or_else(wrong_caller)?;
    let service = dokime_protocol::descriptor_pool().get_service_by_name(service).ok_or_else(wrong_caller)?;
    let method = service.methods().find(|candidate| candidate.name() == method).ok_or_else(wrong_caller)?;
    let declared = harmos_protocol::schema::method_callers(&method).map_err(|_| wrong_caller())?;
    let allowed = matches!(
        (declared, caller),
        (model::Callers::All, model::Callers::Clients | model::Callers::Plugins)
            | (model::Callers::Clients, model::Callers::Clients)
            | (model::Callers::Plugins, model::Callers::Plugins)
    );
    if !allowed {
        return Err(wrong_caller());
    }
    Ok(())
}

/* === Private Functions === */

pub(crate) fn declaration<Q: ReflectMessage + Default, R: ReflectMessage + Default>(
    id: &str, types: FileDescriptorSet,
) -> Result<model::OperationContract> {
    let request = Q::default().descriptor().full_name().to_owned();
    let response = R::default().descriptor().full_name().to_owned();
    let fingerprint =
        harmos_protocol::schema::fingerprint(id, &types, &request, &response).map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(model::OperationContract {
        id: id.into(),
        request,
        response,
        visibility: model::Visibility::Public as i32,
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
