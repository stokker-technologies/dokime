/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod backend;
mod contract;
pub mod launch;
mod owner;

/* === Re-Exports === */
pub use contract::*;
pub use owner::DesktopSession;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct DesktopSessionControl(DesktopSession);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl DesktopSessionControl {
    pub fn active(owner: DesktopSession) -> Self {
        Self(owner)
    }
    pub async fn shutdown(&self) -> Result<(), DesktopSessionError> {
        self.0.shutdown().await?;
        Ok(())
    }
    pub fn status(&self) -> DesktopSessionSnapshot {
        self.0.status()
    }
}

/* === Public Functions === */
pub async fn desktop_session_select(
    state: tauri::State<'_, DesktopSessionControl>,
    selection: BackendSelection,
) -> Result<DesktopSessionSnapshot, DesktopSessionError> {
    state.0.select(selection).await
}

pub async fn desktop_session_disconnect(
    state: tauri::State<'_, DesktopSessionControl>,
) -> Result<DesktopSessionSnapshot, DesktopSessionError> {
    state.0.disconnect().await
}

pub fn desktop_session_status(
    state: tauri::State<'_, DesktopSessionControl>,
) -> Result<DesktopSessionSnapshot, DesktopSessionError> {
    Ok(state.0.status())
}

/* === Private Functions === */

/* === Tests === */
