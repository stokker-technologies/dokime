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

use crate::connection::{ConnectionSession, host::Session};
use crate::native_session::DesktopSessionControl;
use dokime_protocol::client;
use prost::Message;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingsScope {
    Local,
    Backend,
}

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
) -> Result<tauri::ipc::Response, SettingsCommandError> {
    let client = client(&owner, &connection, scope).await?;
    let snapshot: client::SettingsGetResponse =
        client.unary("/dokime.client.v1.Settings/Get", client::SettingsGetRequest::default()).await.map_err(command_error)?;
    let snapshot = with_listener(&client, snapshot).await?;
    Ok(tauri::ipc::Response::new(snapshot.encode_to_vec()))
}

pub async fn desktop_settings_update(
    app: AppHandle, owner: State<'_, DesktopSessionControl>, connection: State<'_, ConnectionSession>, scope: SettingsScope,
    update: Vec<u8>,
) -> Result<tauri::ipc::Response, SettingsCommandError> {
    let client = client(&owner, &connection, scope).await?;
    if update.len() > crate::rpc::protocol::FRAME_BYTES as usize {
        return Err(SettingsCommandError::Validation {
            message: "Save Preferences: The Request Exceeds the Desktop Frame Limit".into(),
        });
    }
    let update = client::SettingsSetRequest::decode(update.as_slice())
        .map_err(|error| SettingsCommandError::Validation { message: error.to_string() })?;
    let current: client::SettingsGetResponse =
        client.unary("/dokime.client.v1.Settings/Get", client::SettingsGetRequest::default()).await.map_err(command_error)?;
    let previous = current.value.as_ref().ok_or_else(|| SettingsCommandError::Validation {
        message: "Read Preferences: Host Omitted Settings".into(),
    })?;
    let proposed = update.value.as_ref().ok_or_else(|| SettingsCommandError::Validation {
        message: "Save Preferences: Supply Settings".into(),
    })?;
    if proposed.listener != previous.listener {
        return Err(SettingsCommandError::Validation {
            message: "Save Preferences: Configure Remote Access Through Its Listener Resource".into(),
        });
    }
    let valid = match scope {
        SettingsScope::Local => {
            proposed.report == previous.report
                && proposed.telemetry == previous.telemetry
                && proposed.listener == previous.listener
                && proposed.plugins == previous.plugins
        }
        SettingsScope::Backend => proposed.appearance == previous.appearance && proposed.client == previous.client,
    };
    if !valid {
        return Err(SettingsCommandError::Validation {
            message: "Save Preferences: Change Only Fields Owned by the Selected Scope".into(),
        });
    }
    let saved: client::SettingsSetResponse = client.unary("/dokime.client.v1.Settings/Set", update).await.map_err(command_error)?;
    let saved = with_listener(
        &client,
        client::SettingsGetResponse {
            value: saved.value,
            revision: saved.revision,
            resolved_at: None,
        },
    )
    .await?;
    if saved.value.as_ref().and_then(|settings| settings.listener.as_ref()).is_some_and(|listener| !listener.enabled)
        && matches!(owner.status().ownership, crate::native_session::BackendOwnership::OwnedLocal)
        && let Err(error) = super::firewall::remove(app).await
    {
        tracing::warn!(%error, "Firewall cleanup could not complete");
    }
    if matches!(scope, SettingsScope::Local)
        && let Some(preferences) = saved.value.as_ref().and_then(|settings| settings.client)
    {
        connection.set_preferences(preferences);
    }
    Ok(tauri::ipc::Response::new(saved.encode_to_vec()))
}

/* === Private Functions === */

async fn client(
    owner: &DesktopSessionControl, connection: &ConnectionSession, scope: SettingsScope,
) -> Result<Session, SettingsCommandError> {
    if matches!(scope, SettingsScope::Local) {
        return owner.local_client().await.map_err(|error| SettingsCommandError::Persistence { message: format!("{error:?}") });
    }
    connection.lease(&connection.status().generation).map(|lease| lease.client()).map_err(|_| SettingsCommandError::Disconnected)
}

// Access Owns Listener State; Application Preferences Only Project Its Current Value.
async fn with_listener(
    client: &Session, mut snapshot: client::SettingsGetResponse,
) -> Result<client::SettingsGetResponse, SettingsCommandError> {
    use dokime_sdk::client::v1::access;
    let listener: access::ListenerGetResponse =
        client.unary("/harmos.v1.access.Listener/Get", access::ListenerGetRequest {}).await.map_err(command_error)?;
    let value = listener.value.ok_or_else(|| SettingsCommandError::Validation {
        message: "Read Preferences: Host Omitted Its Listener Value".into(),
    })?;
    let settings = snapshot.value.as_mut().ok_or_else(|| SettingsCommandError::Validation {
        message: "Read Preferences: Host Omitted Application Settings".into(),
    })?;
    settings.listener = Some(client::RemoteListenerSettings {
        enabled: value.enabled,
        address: value.endpoint,
        certificate_names: value.certificate_names,
        tls_enabled: true,
        authentication_enabled: true,
    });
    Ok(snapshot)
}

fn command_error(error: crate::connection::host::Error) -> SettingsCommandError {
    if let crate::connection::host::Error::Rpc(status) = &error {
        match status.code() {
            tonic::Code::Aborted | tonic::Code::AlreadyExists => {
                return SettingsCommandError::Conflict { message: status.message().into() };
            }
            tonic::Code::InvalidArgument => return SettingsCommandError::Validation { message: status.message().into() },
            tonic::Code::Unavailable => return SettingsCommandError::Disconnected,
            _ => {}
        }
    }
    SettingsCommandError::Persistence { message: error.to_string() }
}

/* === Tests === */
