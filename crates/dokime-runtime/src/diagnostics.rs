/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Diagnostic Runtime Hooks With Shared Portable Values

/* === Modules === */

/* === Re-Exports === */

pub use dokime_domain::model::diagnostics::*;

#[cfg(feature = "native")]
pub use dokime_tracing as test_trace;

/* === Imports === */

// Standard Library Imports
use std::{any::Any, cell::RefCell, sync::Once};

// Third Party Imports

// Local Imports

/* === Constants === */

static HOOK: Once = Once::new();

/* === Macros === */

thread_local! {
    static PANIC: RefCell<Option<Diagnostic>> = const { RefCell::new(None) };
}

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn install_panic_hook() {
    HOOK.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let mut diagnostic = Diagnostic::of(format!("plugin panicked: {}", payload(info.payload())));
            diagnostic.location = info.location().map(|location| format!("{}:{}", location.file(), location.line())).unwrap_or_default();
            // stderr Is Separate From The Sidecar Protocol On stdout.
            eprint!("{diagnostic}");
            PANIC.with(|panicked| panicked.replace(Some(diagnostic)));
        }));
    });
}

#[cfg(feature = "native")]
pub fn install_subscriber() {
    use tracing_subscriber::Layer;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let layers: Vec<dokime_tracing::BoxedLayer> =
        vec![tracing_subscriber::fmt::layer().with_ansi(false).without_time().with_writer(std::io::stderr).with_filter(filter).boxed()];
    let _ = dokime_tracing::configure(layers);
}

pub fn panicked(carried: &(dyn Any + Send)) -> Diagnostic {
    PANIC.with(|panicked| panicked.take()).unwrap_or_else(|| Diagnostic::of(format!("plugin panicked: {}", payload(carried))))
}

/* === Private Functions === */

fn payload(carried: &(dyn Any + Send)) -> String {
    carried
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| carried.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic with no message".to_owned())
}

/* === Tests === */
