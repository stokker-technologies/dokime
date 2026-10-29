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
use std::{collections::BTreeMap, process::Command};

// Third Party Imports
use starship_battery::{
    Battery, Manager as BatteryManager, State as BatteryState,
    units::{
        electric_potential::volt,
        energy::watt_hour,
        power::watt,
        ratio::{percent, ratio},
        thermodynamic_temperature::degree_celsius,
        time::second,
    },
};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, System};

// Local Imports
use crate::{
    HostStreamSample, HostTelemetryStream, Result,
    collectors::{integer, maybe_insert},
};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SystemCollector {
    sample_index: u64,
    system: System,
    components: Components,
    battery_manager: Option<BatteryManager>,
}

#[derive(Default)]
struct PowerProfileContext {
    plan_name: Option<String>,
    plan_guid: Option<String>,
    mode: Option<String>,
    performance_bias: Option<i64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for SystemCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemCollector {
    pub fn new() -> Self {
        Self {
            sample_index: 0,
            system: System::new_all(),
            components: Components::new_with_refreshed_list(),
            battery_manager: BatteryManager::new().ok(),
        }
    }

    pub fn collect(&mut self) -> Result<HostStreamSample> {
        self.sample_index += 1;
        self.system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
        self.system.refresh_memory_specifics(MemoryRefreshKind::everything());
        self.components.refresh(true);

        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_pressure = if total_memory > 0 { Some((used_memory as f64 / total_memory as f64) * 100.0) } else { None };
        let temperatures = self
            .components
            .list()
            .iter()
            .filter_map(|component| component.temperature())
            .filter(|temperature| temperature.is_finite())
            .map(f64::from)
            .collect::<Vec<_>>();
        let battery = first_battery(self.battery_manager.as_ref());
        let power = power_profile_context();
        let first_cpu = self.system.cpus().first();

        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("probe_status".to_owned(), Value::from("ok")),
            ("cpu_load_percent".to_owned(), Value::from(f64::from(self.system.global_cpu_usage()))),
            ("memory_total_bytes".to_owned(), integer(total_memory)),
            ("memory_used_bytes".to_owned(), integer(used_memory)),
            ("system_uptime_s".to_owned(), Value::from(System::uptime() as f64)),
            ("temperature_sensor_count".to_owned(), Value::Integer(temperatures.len() as i64)),
            ("cpu_architecture".to_owned(), Value::from(std::env::consts::ARCH)),
            ("logical_cpu_count".to_owned(), Value::Integer(self.system.cpus().len() as i64)),
        ]);
        maybe_insert(&mut values, "physical_cpu_count", System::physical_core_count().map(|count| Value::Integer(count as i64)));
        maybe_insert(&mut values, "cpu_name", first_cpu.map(|cpu| cpu.name().trim()).filter(|value| !value.is_empty()).map(Value::from));
        maybe_insert(&mut values, "cpu_brand", first_cpu.map(|cpu| cpu.brand().trim()).filter(|value| !value.is_empty()).map(Value::from));
        maybe_insert(
            &mut values,
            "cpu_vendor_id",
            first_cpu.map(|cpu| cpu.vendor_id().trim()).filter(|value| !value.is_empty()).map(Value::from),
        );
        maybe_insert(&mut values, "cpu_frequency_mhz", first_cpu.map(|cpu| Value::Integer(cpu.frequency() as i64)));
        maybe_insert(&mut values, "host_name", System::host_name().map(Value::from));
        maybe_insert(&mut values, "user", std::env::var("USERNAME").or_else(|_| std::env::var("USER")).ok().map(Value::from));
        maybe_insert(&mut values, "git_user_name", git_config_value("user.name").map(Value::from));
        maybe_insert(&mut values, "git_user_email", git_config_value("user.email").map(Value::from));
        maybe_insert(&mut values, "os_name", System::name().map(Value::from));
        maybe_insert(&mut values, "os_version", System::long_os_version().map(Value::from));
        maybe_insert(&mut values, "kernel_version", System::kernel_version().map(Value::from));
        maybe_insert(&mut values, "memory_pressure_percent", memory_pressure.map(Value::from));
        maybe_insert(&mut values, "max_temperature_c", temperatures.into_iter().reduce(f64::max).map(Value::from));
        for (key, value) in battery_values(battery.as_ref()) {
            maybe_insert(&mut values, key, value);
        }
        maybe_insert(&mut values, "power_plan_name", power.plan_name.map(Value::from));
        maybe_insert(&mut values, "power_plan_guid", power.plan_guid.map(Value::from));
        maybe_insert(&mut values, "power_mode", power.mode.map(Value::from));
        maybe_insert(&mut values, "power_mode_performance_bias", power.performance_bias.map(Value::from));

        Ok(HostStreamSample::new(HostTelemetryStream::System, values))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn git_config_value(key: &str) -> Option<String> {
    let mut command = Command::new("git");
    command.args(["config", "--get", key]);
    let output = super::run_command_bounded(command, system_probe_timeout()).ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

// A System Probe Is Bounded By The Collection It Serves.
fn system_probe_timeout() -> std::time::Duration {
    crate::collection_timeout(HostTelemetryStream::System.as_str())
}

fn first_battery(manager: Option<&BatteryManager>) -> Option<Battery> {
    manager?.batteries().ok()?.next()?.ok()
}

fn battery_values(battery: Option<&Battery>) -> Vec<(&'static str, Option<Value>)> {
    let Some(battery) = battery else {
        return Vec::new();
    };
    let state = battery.state();
    let number = |value: f32| value.is_finite().then(|| Value::from(f64::from(value)));
    let text = |value: Option<&str>| value.map(Value::from);

    vec![
        ("battery_percent", number(battery.state_of_charge().get::<percent>())),
        ("battery_state", Some(Value::from(state.to_string()))),
        ("battery_power_plugged", Some(Value::from(matches!(state, BatteryState::Charging | BatteryState::Full)))),
        ("battery_temperature_c", battery.temperature().and_then(|temperature| number(temperature.get::<degree_celsius>()))),
        ("battery_vendor", text(battery.vendor())),
        ("battery_model", text(battery.model())),
        ("battery_serial_number", text(battery.serial_number())),
        ("battery_technology", Some(Value::from(battery.technology().to_string()))),
        ("battery_state_of_charge_ratio", number(battery.state_of_charge().get::<ratio>())),
        ("battery_energy_wh", number(battery.energy().get::<watt_hour>())),
        ("battery_energy_full_wh", number(battery.energy_full().get::<watt_hour>())),
        ("battery_energy_full_design_wh", number(battery.energy_full_design().get::<watt_hour>())),
        ("battery_energy_rate_w", number(battery.energy_rate().get::<watt>())),
        ("battery_voltage_v", number(battery.voltage().get::<volt>())),
        ("battery_state_of_health_ratio", number(battery.state_of_health().get::<ratio>())),
        ("battery_time_to_full_s", battery.time_to_full().and_then(|time| number(time.get::<second>()))),
        ("battery_time_to_empty_s", battery.time_to_empty().and_then(|time| number(time.get::<second>()))),
        ("battery_cycle_count", battery.cycle_count().map(|count| Value::Integer(i64::from(count)))),
    ]
}

fn power_profile_context() -> PowerProfileContext {
    #[cfg(windows)]
    {
        let mut command = Command::new("powercfg");
        command.arg("/getactivescheme");
        let Ok(output) = super::run_command_bounded(command, system_probe_timeout()) else {
            return PowerProfileContext::default();
        };
        if !output.status.success() {
            return PowerProfileContext::default();
        }
        let text = String::from_utf8_lossy(&output.stdout);
        parse_windows_power_scheme(&text)
    }

    #[cfg(not(windows))]
    {
        PowerProfileContext::default()
    }
}

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(windows)]
fn power_performance_bias(name: &str) -> i64 {
    match power_mode_label(name).as_str() {
        "best_performance" => 100,
        "balanced" => 50,
        "best_efficiency" => 0,
        _ => 50,
    }
}

/* === Tests === */
