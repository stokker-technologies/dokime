/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Interval Settings Decide the Placement the Built-In Host Starts With

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroU32, time::Duration};

// Third Party Imports

// Local Imports
use dokime_gui::native_session::backend::Backend;
use dokime_gui::settings::{self, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn edited_interval_settings_reach_the_host_placement() {
    // Nothing Saved Starts the Host With the Runtime Defaults
    let application = tempfile::tempdir().unwrap();
    let file = settings::path(application.path());
    let backend = Backend::new(application.path().join("store")).with_settings(file.clone());
    let placement = backend.settings().host.placement;
    assert_eq!(placement.interval_window, dokime_runtime::INTERVAL_WINDOW);
    assert_eq!(placement.interval_lag, dokime_runtime::INTERVAL_LAG);

    // Saved Edits Are the Placement the Next Host Start Uses
    Store::new(file.clone())
        .update(|saved| {
            saved.interval_window_ms = 250;
            saved.interval_lag_windows = NonZeroU32::new(3).unwrap();
            Ok(())
        })
        .unwrap();
    let placement = backend.settings().host.placement;
    assert_eq!(placement.interval_window, Duration::from_millis(250));
    assert_eq!(placement.interval_lag.get(), 3);

    // An Unreadable File Starts the Host With the Defaults Instead of Refusing It
    std::fs::write(&file, b"{").unwrap();
    let placement = backend.settings().host.placement;
    assert_eq!(placement.interval_window, dokime_runtime::INTERVAL_WINDOW);
    assert_eq!(placement.interval_lag, dokime_runtime::INTERVAL_LAG);
}
