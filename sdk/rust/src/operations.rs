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
use std::time::Duration;

// Third Party Imports
use ::harmos::plugin;
use prost_types::Timestamp;

// Local Imports
use crate::context::{self, Executions};
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
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
