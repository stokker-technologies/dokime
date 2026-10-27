/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Scoped Application Settings Commands

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Serialize;
use tauri::State;

// Local Imports
use crate::connection::{ConnectionSession, registry};
use crate::native_session::DesktopSessionControl;
use dokime::domain::model::settings::application::{SettingsScope, SettingsSnapshot, UpdateSettings};
use dokime::operations::settings::{ReadApplicationSettings, UpdateApplicationSettings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SettingsCommandError {
    Validation { message: String },
    Conflict { message: String },
    Disconnected,
    Persistence { message: String },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn desktop_settings_read(
    owner: State<'_, DesktopSessionControl>, connection: State<'_, ConnectionSession>, scope: SettingsScope,
) -> Result<SettingsSnapshot, SettingsCommandError> {
    let client = client(&owner, &connection, scope).await?;
    client.settings_read_application_settings(ReadApplicationSettings {}).await.map_err(command_error)
}

pub async fn desktop_settings_update(
    owner: State<'_, DesktopSessionControl>, connection: State<'_, ConnectionSession>, scope: SettingsScope, update: UpdateSettings,
) -> Result<SettingsSnapshot, SettingsCommandError> {
    let client = client(&owner, &connection, scope).await?;
    let current = client.settings_read_application_settings(ReadApplicationSettings {}).await.map_err(command_error)?;
    update.settings.validate_scope(&current.settings, scope).map_err(|message| SettingsCommandError::Validation { message })?;
    let saved = client.settings_update_application_settings(UpdateApplicationSettings { update }).await.map_err(command_error)?;
    if matches!(scope, SettingsScope::Local) {
        connection.set_preferences(saved.settings.client);
    }
    Ok(saved)
}

/* === Private Functions === */

async fn client(
    owner: &DesktopSessionControl, connection: &ConnectionSession, scope: SettingsScope,
) -> Result<dokime::client::Client, SettingsCommandError> {
    if matches!(scope, SettingsScope::Local) {
        return owner
            .local_client()
            .await
            .map_err(|error| SettingsCommandError::Persistence { message: format!("{:?}", registry::session_error(error)) });
    }
    let status = connection.status();
    connection.lease(&status.generation).map(|lease| lease.client()).map_err(|_| SettingsCommandError::Disconnected)
}

fn command_error(error: dokime::client::Error) -> SettingsCommandError {
    if let dokime::client::Error::ExecutionRejected { rejection } = &error {
        if rejection.code == "settings_conflict" {
            return SettingsCommandError::Conflict { message: rejection.message.clone() };
        }
        if rejection.code == "invalid_settings" {
            return SettingsCommandError::Validation { message: rejection.message.clone() };
        }
    }
    if let dokime::client::Error::Rpc(status) = &error {
        match status.code() {
            tonic::Code::Aborted | tonic::Code::AlreadyExists => {
                return SettingsCommandError::Conflict { message: status.message().into() };
            }
            tonic::Code::InvalidArgument => return SettingsCommandError::Validation { message: status.message().into() },
            tonic::Code::Unavailable => return SettingsCommandError::Disconnected,
            _ => {}
        }
    }
    if matches!(error, dokime::client::Error::Closed | dokime::client::Error::BackendUnavailable) {
        return SettingsCommandError::Disconnected;
    }
    SettingsCommandError::Persistence { message: error.to_string() }
}

/* === Tests === */
