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

mod schema;

#[cfg(feature = "sidecar")]
mod listeners;

#[cfg(feature = "sidecar")]
mod publisher;
pub mod registration;
pub mod settings;

#[cfg(feature = "sidecar")]
pub mod testing;

/* === Re-Exports === */

pub use configuration::{Configuration, Secret};
pub use schema::schema_defaults;
pub use settings::NoSettings;

#[cfg(feature = "sidecar")]
pub use listeners::{Delivery, HearFuture, Hearing, LISTENER_POLICY, Listener, ListenerContext};

#[cfg(feature = "sidecar")]
pub use publisher::Publisher;

#[cfg(feature = "sidecar")]
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

#[cfg(feature = "sidecar")]
pub fn run<P>() -> std::io::Result<()>
where
    P: crate::Plugin<Context = crate::Supervisor> + harmos::plugin::Declared,
{
    install_panic_hook();
    install_subscriber();
    let capacity = std::num::NonZeroUsize::new(128).expect("native invocation capacity is positive");
    crate::process::block_on(async move {
        let settings = schema_defaults();
        harmos::sidecar::run_plugin_with_settings::<P>(capacity, &settings).await
    })?
}

#[cfg(feature = "sidecar")]
#[track_caller]
pub fn refusal<T, E>(answered: Result<T, E>) -> Result<T, crate::Refusal>
where
    E: std::error::Error + 'static,
{
    let location = std::panic::Location::caller();
    answered.map_err(|error| {
        let diagnostic = crate::diagnostics::Diagnostic::at(location, &error).redacted();
        tracing::error!("{diagnostic}");
        let missing =
            std::iter::successors(Some(&error as &(dyn std::error::Error + 'static)), |source| source.source()).find_map(|source| {
                match source.downcast_ref::<crate::Error>() {
                    Some(crate::Error::UnavailableSecret(secret)) => Some(secret.as_str()),
                    _ => None,
                }
            });
        let failure =
            harmos::domain::Failure::originate(harmos::operations::FailureKind::Permanent, &error, harmos::domain::failure::Origin::Plugin);
        match missing {
            Some(secret) => crate::Refusal::failed(
                dokime_domain::model::plugins::error::MISSING_SECRET_CODE,
                failure.with_detail("dokime.secret", secret),
            ),
            None => crate::Refusal::failed(failure.refusal_code().unwrap_or(harmos::domain::FAILED), failure),
        }
    })
}

/* === Private Functions === */

/* === Tests === */
