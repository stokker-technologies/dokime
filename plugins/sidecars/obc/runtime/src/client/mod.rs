/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::{Error, app::Handle, device::Device, transactions::SetPower};

#[derive(Clone)]
pub struct Client {
    handle: Handle,
}

impl Client {
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }

    pub async fn ensure_ready(&self) -> Result<(), Error> {
        if self.stopped().await? { Err(Error::Stopped) } else { Ok(()) }
    }

    pub async fn stopped(&self) -> Result<bool, Error> {
        let state = self.handle.state.state();
        let stopped = state.read(|state| state.stopped).await?;
        Ok(stopped.value)
    }

    pub async fn device(&self) -> Result<Device, Error> {
        let state = self.handle.state.state();
        let device = state.read(|state| state.device.clone()).await?;
        Ok(device.value)
    }

    pub async fn set_power(&self, enabled: bool) -> Result<(), dokime::harmos::Error<Error>> {
        self.handle.state.commit(SetPower { enabled }).await?;
        Ok(())
    }
}
