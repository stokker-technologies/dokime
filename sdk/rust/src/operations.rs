/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation Authoring Context

/* === Modules === */

/* === Re-Exports === */

pub use crate::operation::Operation;
pub use ::harmos::plugin::StateHandle;

/* === Imports === */

// Standard Library Imports
use std::{collections::HashSet, sync::Arc, time::Duration};

// Third Party Imports
use ::harmos::plugin;
use prost_types::Timestamp;

// Local Imports
use crate::context::{self, Executions};
use crate::templates::Environment;
use crate::{Message, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// An Operation Cannot Manufacture an Execution Context.
pub struct Context {
    inner: context::Context,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Context {
    /// Bind a Library Collection Invocation to the Same Author State as the Plugin.
    pub fn from_collection<T: Send + Sync + 'static>(transport: &plugin::Context, state: T) -> Result<Self> {
        let executions = Executions::new(Arc::new(state), HashSet::new());
        Self::invocation(transport, &executions)
    }

    pub(crate) fn from_inner(inner: context::Context) -> Self {
        Self { inner }
    }
    pub(crate) fn invocation(transport: &plugin::Context, executions: &Executions) -> Result<Self> {
        Ok(Self {
            inner: context::Context::invocation(transport, executions)?,
        })
    }
    pub fn plugin<T: Send + Sync + 'static>(&self) -> Result<&T> {
        self.inner.plugin::<T>()
    }
    pub fn state(&self) -> Result<StateHandle> {
        self.inner.state()
    }
    pub fn attempt(&self) -> u32 {
        self.inner.attempt()
    }
    pub fn stop_requested(&self) -> bool {
        self.inner.stop_requested()
    }
    pub async fn sleep(&self, duration: Duration) -> Result<()> {
        self.inner.sleep(duration).await
    }
    pub async fn now(&self) -> Result<Timestamp> {
        self.inner.now().await
    }
    pub async fn cancelled(&self) -> String {
        self.inner.cancelled().await
    }
    pub async fn secret(&self, identifier: &str) -> Result<Vec<u8>> {
        self.inner.secret(identifier).await
    }
    pub async fn settings<T: Message>(&self) -> Result<T> {
        self.inner.settings::<T>().await
    }
    pub async fn setting(&self, identifier: &str) -> Result<crate::SettingValue> {
        self.inner.setting(identifier).await
    }
    pub async fn duration_setting(&self, identifier: &str) -> Result<Duration> {
        self.inner.duration_setting(identifier).await
    }
    pub async fn environment(&self) -> Result<Environment> {
        self.inner.environment().await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
