/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::{RuntimeHandle, runtime::Result};
use dokime_domain::model::settings::{SettingsSnapshot, UpdateSettings};

impl RuntimeHandle {
    pub async fn read_application_settings(&self) -> Result<SettingsSnapshot> {
        Ok(self.local_storage.read_application_settings().await?)
    }

    pub async fn update_application_settings(&self, update: UpdateSettings) -> Result<SettingsSnapshot> {
        if self.is_shutting_down() {
            return Err(crate::runtime::Error::ShuttingDown);
        }
        Ok(self.local_storage.update_application_settings(update).await?)
    }
}
