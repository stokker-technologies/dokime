/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Interface Composition Within The Application Lifecycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use futures_util::future::BoxFuture;

// Local Imports
use crate::{RuntimeHandle, app::Dokime, runtime::Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type LiveRuntime = Arc<tokio::sync::RwLock<Option<RuntimeHandle>>>;
pub type Prepare = dyn Fn(&mut harmos::Setup<Dokime>, &LiveRuntime, &crate::config::RuntimeConfig) -> Result<()> + Send + Sync;
pub type Initialize = dyn Fn(RuntimeHandle) -> BoxFuture<'static, Result<()>> + Send + Sync;
pub type Cleanup = dyn Fn() -> BoxFuture<'static, Result<()>> + Send + Sync;

/* === Structs === */

#[derive(Clone)]
pub struct Composition {
    pub prepare: Arc<Prepare>,
    pub initialize: Arc<Initialize>,
    pub cleanup: Arc<Cleanup>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for Composition {
    fn default() -> Self {
        Self {
            prepare: Arc::new(|_, _, _| Ok(())),
            initialize: Arc::new(|_| Box::pin(async { Ok(()) })),
            cleanup: Arc::new(|| Box::pin(async { Ok(()) })),
        }
    }
}

/* === Public Functions === */

pub async fn live_runtime(live: &LiveRuntime) -> std::result::Result<RuntimeHandle, tonic::Status> {
    live.read().await.clone().ok_or_else(|| tonic::Status::unavailable("runtime is not active"))
}

/* === Private Functions === */

/* === Tests === */
