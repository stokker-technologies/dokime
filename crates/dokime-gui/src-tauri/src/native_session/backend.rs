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

use super::{BackendDiagnostics, DesktopSessionError as Error};
use crate::connection::host::{self, Connection};
use std::path::PathBuf;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct Backend {
    pub root: PathBuf,
    pub log: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Backend {
    pub fn new(root: PathBuf) -> Self {
        Self { root, log: None }
    }

    pub fn with_log(mut self, log: Option<PathBuf>) -> Self {
        self.log = log;
        self
    }
}

/* === Public Functions === */

pub async fn start(backend: &Backend, mut on_phase: impl FnMut(&str) + Send) -> Result<(Connection, String), Error> {
    on_phase("starting");
    let mut connection = Connection::new(host::desktop_settings()).map_err(|error| failure(backend, error))?;
    let session = connection.start(&backend.root).await.map_err(|error| failure(backend, error))?;
    tracing::info!(endpoint = session.endpoint(), root = %backend.root.display(), "Execution Host Ready");
    Ok((connection, session.endpoint().to_owned()))
}

/* === Private Functions === */

fn failure(backend: &Backend, error: host::Error) -> Error {
    if let host::Error::Host(dokime_runtime_next::Error::OccupiedRoot(root)) = error {
        let message = format!(
            "The data folder {} is in use by another host. Connect to that host, close its owner or select a fresh storage root.",
            root.display()
        );
        return Error::BackendRunning { root, message };
    }
    let message = format!("Start Local Host at {}: {error}", backend.root.display());
    Error::Startup {
        message: message.clone(),
        diagnostics: Some(Box::new(BackendDiagnostics {
            cause: message,
            log: backend.log.as_ref().map(|path| path.display().to_string()),
        })),
    }
}

/* === Tests === */
