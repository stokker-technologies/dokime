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

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use ::harmos::plugin;
use prost_types::Timestamp;

// Local Imports
use crate::context::Executions;
use crate::{Message, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// An Operation Cannot Manufacture an Execution Context.
pub struct Context {
    inner: crate::context::Context,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Context {
    pub(crate) fn from_inner(inner: crate::context::Context) -> Self {
        Self { inner }
    }
    pub(crate) fn invocation(transport: &plugin::Context, executions: &Executions) -> Result<Self> {
        Ok(Self {
            inner: crate::context::Context::invocation(transport, executions)?,
        })
    }
    pub fn plugin<T: Send + Sync + 'static>(&self) -> Result<&T> {
        self.inner.plugin::<T>()
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
