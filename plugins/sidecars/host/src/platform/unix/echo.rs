/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::time::Duration;

// ping Waits For The Echo; Its Process Is Bounded By The Collection It Serves.
// The Round Trip In Milliseconds, Or None When No Reply Arrived Within The Wait.
pub fn round_trip_ms(host: &str, timeout: Duration) -> Result<Option<f64>, String> {
    let mut command = dokime_domain::process::command("ping");
    command.args(["-c", "1", "-W", &timeout.as_secs().max(1).to_string(), host]);
    let bound = crate::host::wifi_latency::POLICY.attempt_timeout;
    let output = crate::collectors::run_command_bounded(command, bound).map_err(|error| format!("ping command failed: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{stdout}\n{stderr}");

    // ping Exits 1 When No Reply Arrived And 2 When It Could Not Probe.
    match output.status.code() {
        Some(0) => {}
        Some(1) => return Ok(None),
        _ => return Err(output_text.trim().to_owned()),
    }

    let latency = parse_ping_latency_ms(&output_text).ok_or_else(|| format!("ping latency was not found in command output for {host}"))?;
    Ok(Some(latency))
}

fn parse_ping_latency_ms(output: &str) -> Option<f64> {
    let lower = output.to_ascii_lowercase();
    for marker in ["time=", "time<"] {
        let Some(index) = lower.find(marker) else {
            continue;
        };
        let number = leading_number(&lower[index + marker.len()..]);
        let value = number.parse::<f64>().ok()?;
        return Some(if marker == "time<" { value.max(0.5) } else { value });
    }
    None
}

fn leading_number(text: &str) -> String {
    text.trim_start().chars().take_while(|value| value.is_ascii_digit() || *value == '.').collect()
}
