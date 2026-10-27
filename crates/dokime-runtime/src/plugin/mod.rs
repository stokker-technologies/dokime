/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod configuration;

#[cfg(feature = "native")]
mod listeners;

#[cfg(feature = "native")]
mod publisher;
pub mod registration;
pub mod settings;

#[cfg(feature = "native")]
pub mod testing;

/* === Re-Exports === */

pub use configuration::{Configuration, Secret};
pub use settings::NoSettings;

#[cfg(feature = "native")]
pub use listeners::{Delivery, Heard, LISTENER_POLICY, Listener};

#[cfg(feature = "native")]
pub use publisher::Publisher;

#[cfg(feature = "native")]
pub use crate::diagnostics::install_subscriber;
pub use crate::diagnostics::{Diagnostic, redacted};
pub use crate::diagnostics::{captured, install_panic_hook, panicked};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[cfg(feature = "native")]
pub fn run<P>() -> std::io::Result<()>
where
    P: crate::Plugin<Context = crate::Supervisor> + harmos::plugin::Declared,
{
    install_panic_hook();
    install_subscriber();
    let capacity = std::num::NonZeroUsize::new(128).expect("native invocation capacity is positive");
    crate::process::block_on(harmos::sidecar::run_plugin::<P>(capacity))?
}

#[cfg(feature = "native")]
#[track_caller]
pub fn refusal<T, E>(answered: Result<T, E>) -> Result<T, crate::Refusal>
where
    E: std::error::Error,
{
    captured(answered).map_err(|diagnostic| {
        tracing::error!("{diagnostic}");
        crate::Refusal::of(diagnostic.summary())
    })
}

/* === Private Functions === */

/* === Tests === */
