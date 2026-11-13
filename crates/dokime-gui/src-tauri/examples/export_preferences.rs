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
use std::path::PathBuf;

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_gui::native_session::backend::{self, Backend};
use dokime_sdk::{client::v1::access, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<(), String> {
    // The Generated Host Router's Debug Future Exceeds the Windows Main Stack.
    std::thread::Builder::new()
        .name("Export Preferences".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| tokio::runtime::Runtime::new().map_err(|error| error.to_string())?.block_on(export()).map_err(|error| error.to_string()))
        .map_err(|error| error.to_string())?
        .join()
        .map_err(|_| "Export Preferences: Worker Panicked".to_owned())?
}

async fn export() -> Result<(), Box<dyn std::error::Error>> {
    // Reuse the Generator-Owned Host Root Without Resetting Its Storage.
    let directory = PathBuf::from(std::env::args_os().nth(1).ok_or("Export Preferences: Supply an Output Directory")?);
    let (mut owner, _) =
        backend::start(&Backend::new(directory.join("host")), |_| {}).await.map_err(|error| format!("Export Preferences: {error:?}"))?;
    let session = owner.session()?;
    let result = async {
        // Read Product Defaults and the Independently Owned Listener.
        let mut snapshot: v1::SettingsSnapshot =
            session.unary("/dokime.v1.Settings/ReadApplicationSettings", v1::SettingsReadApplicationSettingsRequest {}).await?;
        let listener: access::ListenerGetResponse = session.unary("/harmos.v1.access.Listener/Get", access::ListenerGetRequest {}).await?;
        let listener = listener.value.ok_or("Export Preferences: Host Omitted Listener")?;
        snapshot.settings.as_mut().ok_or("Export Preferences: Host Omitted Settings")?.listener = Some(v1::RemoteListenerSettings {
            enabled: listener.enabled,
            address: listener.endpoint,
            certificate_names: listener.certificate_names,
            tls_enabled: true,
            authentication_enabled: true,
        });
        std::fs::create_dir_all(&directory)?;
        std::fs::write(directory.join("defaults.bin"), snapshot.encode_to_vec())?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    owner.close().await?;
    result
}

/* === Tests === */
