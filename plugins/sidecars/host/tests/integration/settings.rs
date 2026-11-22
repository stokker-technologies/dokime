/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Setting Stories: CPU Load Relations Beyond Library Scalar Bounds

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::Error;
use plugin_host::settings::CpuLoad;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn defaults() -> CpuLoad {
    CpuLoad {
        period: Duration::from_millis(250),
        sample_period: Duration::from_millis(250),
        slice: Duration::from_millis(10),
        proportional_gain: 0.5,
        integral_gain: 0.5,
        integral_min: -1.0,
        integral_max: 1.0,
        settlement: Duration::from_secs(2),
        batch_iterations: 4_096,
    }
}

/* === Tests === */

#[test]
fn cpu_load_settings_keep_periods_slices_and_integrator_bounds_consistent() -> dokime::Result<()> {
    defaults().validate()?;
    let refused = [
        CpuLoad {
            sample_period: Duration::from_millis(251),
            ..defaults()
        },
        CpuLoad {
            slice: Duration::from_millis(251),
            ..defaults()
        },
        CpuLoad {
            settlement: Duration::from_millis(9),
            ..defaults()
        },
        CpuLoad {
            integral_min: 0.0,
            integral_max: 0.0,
            ..defaults()
        },
    ];
    for load in refused {
        assert!(matches!(load.validate(), Err(Error::Invalid(_))), "{load:?}");
    }
    Ok(())
}
