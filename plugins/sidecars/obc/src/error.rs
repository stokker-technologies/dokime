/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the operator declined pulse capture")]
    ConfirmationDeclined,

    #[error("pulse sample {index} carries no numeric `{column}`")]
    Unmeasured { index: usize, column: &'static str },

    #[error("missing criterion evidence `{0}`")]
    MissingEvidence(String),

    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime::Error),

    #[error("host operation: {0}")]
    Host(#[from] dokime::harmos::sidecar::CallError),

    #[error("device: {0}")]
    Device(#[from] obc_runtime::Error),

    #[error("testbench launch: {0}")]
    Launch(#[from] obc_runtime::LaunchError),

    #[error("device measurement resource is poisoned")]
    DeviceResourcePoisoned,

    #[error("waiting for a unit to come online: {0}")]
    Awaited(#[from] dokime::prelude::Elapsed),

    #[error("reference image: {0}")]
    Image(String),

    #[error("the case asks for an observation this bench cannot run: {0}")]
    Observation(String),

    #[error("could not encode summary artifact: {0}")]
    Summary(#[from] serde_json::Error),
}

/* === Traits === */

/* === Implementations === */

impl From<Error> for dokime::domains::ActionDispatcherError {
    fn from(error: Error) -> Self {
        Self::Refused {
            code: -32_000,
            message: error.to_string(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
