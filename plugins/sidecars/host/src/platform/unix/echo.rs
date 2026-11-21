/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// ICMP Echo Through ping, Bounded by the Collection It Serves

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::platform::command::{bounded_output, command};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// The Round Trip in Milliseconds, or None When No Reply Arrived Within the Wait.
pub fn round_trip_ms(host: &str, reply_timeout: Duration, payload_bytes: u32, process_timeout: Duration) -> Result<Option<f64>, String> {
    let mut command = command("ping");
    command.args(["-c", "1", "-W", &reply_timeout.as_millis().div_ceil(1_000).to_string(), "-s", &payload_bytes.to_string(), host]);
    let output = bounded_output(command, process_timeout).map_err(|error| format!("Start ping: {error}; Install ping"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{stdout}\n{stderr}");
    // ping Exits 1 When No Reply Arrived and 2 When It Could Not Probe.
    match output.status.code() {
        Some(0) => {}
        Some(1) => return Ok(None),
        _ => return Err(output_text.trim().to_owned()),
    }
    let latency = parse_ping_latency_ms(&output_text).ok_or_else(|| format!("Read ping Output: No Latency for {host}"))?;
    Ok(Some(latency))
}

/// The Latency ping Printed; "time<1" Reports Half a Millisecond at Least.
pub fn parse_ping_latency_ms(output: &str) -> Option<f64> {
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

/* === Private Functions === */

fn leading_number(text: &str) -> String {
    text.trim_start().chars().take_while(|value| value.is_ascii_digit() || *value == '.').collect()
}

/* === Tests === */
