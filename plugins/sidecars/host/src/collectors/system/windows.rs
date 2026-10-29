/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use super::*;

/* === Implementations === */

impl SystemCollector {
    pub(super) fn power_profile_context() -> PowerProfileContext {
        let mut command = Command::new("powercfg");
        command.arg("/getactivescheme");
        let Ok(output) = crate::collectors::run_command_bounded(command, system_probe_timeout()) else {
            return PowerProfileContext::default();
        };
        if !output.status.success() {
            return PowerProfileContext::default();
        }
        let text = String::from_utf8_lossy(&output.stdout);
        parse_windows_power_scheme(&text)
    }
}

fn parse_windows_power_scheme(text: &str) -> PowerProfileContext {
    let plan_guid = text
        .split_whitespace()
        .find(|part| part.chars().filter(|character| *character == '-').count() == 4)
        .map(|part| part.trim_matches(['(', ')']).to_owned());
    let plan_name = text
        .split_once('(')
        .and_then(|(_, tail)| tail.split_once(')').map(|(name, _)| name.trim().to_owned()))
        .filter(|value| !value.is_empty());
    let mode = plan_name.as_deref().map(power_mode_label);
    let performance_bias = plan_name.as_deref().map(power_performance_bias);

    PowerProfileContext {
        plan_name,
        plan_guid,
        mode,
        performance_bias,
    }
}

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
