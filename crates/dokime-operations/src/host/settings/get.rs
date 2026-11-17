/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::OperationContract;

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::RemoteListenerSettings;
use dokime_protocol::host::SettingsGetRequest;
use dokime_protocol::host::SettingsGetResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SettingsGetRequest, SettingsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SettingsGetRequest) -> Result<SettingsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async { crate::host::settings::preferences::read(context, request.at).await })
        .await
}

/// Compose Declared Defaults Without Opening Storage, Credentials or Transports.
pub fn initial_preferences() -> SettingsGetResponse {
    let mut snapshot = crate::host::settings::preferences::initial();
    snapshot.value.as_mut().expect("declared application settings").listener = Some(RemoteListenerSettings {
        enabled: false,
        address: "127.0.0.1:0".into(),
        certificate_names: vec!["localhost".into(), "127.0.0.1".into()],
        tls_enabled: true,
        authentication_enabled: true,
    });
    snapshot
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SettingsGetRequest, SettingsGetResponse, context, "dokime.host.v1", "Settings", "Get");
