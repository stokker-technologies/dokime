/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};

use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.remote.update_listener";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::RemoteUpdateListenerRequest, v1::RemoteListenerStatus>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::RemoteUpdateListenerRequest) -> Result<v1::RemoteListenerStatus> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let settings = request.settings.ok_or_else(|| Error::Invalid("Configure Remote Access: Supply Listener Settings".into()))?;
        let host = context.host.clone();
        tokio::spawn(async move { host.update_remote_listener(settings).await })
            .await
            .map_err(|_| crate::Error::Invalid("Configure Remote Access: Refresh Listener Status Before Retrying".into()))?
    })
    .await
}

pub fn validate_settings(settings: &v1::RemoteListenerSettings) -> Result<std::net::SocketAddr> {
    let invalid = |message: &str| Error::Invalid(format!("Configure Remote Access: {message}"));
    let address = settings.address.parse::<std::net::SocketAddr>().map_err(|_| invalid("Supply a Listener IP Address and Port"))?;
    if settings.enabled && (!settings.tls_enabled || !settings.authentication_enabled) {
        return Err(invalid("Enable TLS and Authentication Before Enabling Remote Access"));
    }
    if settings.certificate_names.is_empty()
        || settings.certificate_names.len() > 64
        || settings.certificate_names.iter().any(|name| name.is_empty() || name.len() > 253)
    {
        return Err(invalid("Supply 1 to 64 Valid Certificate Names"));
    }
    rcgen::CertificateParams::new(settings.certificate_names.clone())
        .map_err(|_| invalid("Supply Valid Certificate DNS Names or IP Addresses"))?;
    if !address.ip().is_unspecified() && !settings.certificate_names.iter().any(|name| name == &address.ip().to_string()) {
        return Err(invalid("Include the Listener IP Address in the Certificate Names"));
    }
    Ok(address)
}

/* === Private Functions === */

/* === Tests === */
