/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SystemCollector {
    pub(super) fn power_profile_context() -> PowerProfileContext {
        let Some((guid, name)) = crate::platform::windows::power::active_scheme() else {
            return PowerProfileContext::default();
        };
        PowerProfileContext {
            mode: name.as_deref().map(power_mode_label),
            performance_bias: name.as_deref().map(power_performance_bias),
            plan_name: name,
            plan_guid: Some(guid),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn power_mode_label(name: &str) -> String {
    let normalized = name.to_ascii_lowercase();
    if normalized.contains("ultimate") || normalized.contains("high performance") {
        "best_performance".to_owned()
    } else if normalized.contains("power saver") || normalized.contains("energy saver") {
        "best_efficiency".to_owned()
    } else if normalized.contains("balanced") {
        "balanced".to_owned()
    } else {
        "unknown".to_owned()
    }
}

fn power_performance_bias(name: &str) -> i64 {
    match power_mode_label(name).as_str() {
        "best_performance" => 100,
        "balanced" => 50,
        "best_efficiency" => 0,
        _ => 50,
    }
}

/* === Tests === */
