/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(all(feature = "runtime", unix))]
mod unix;

#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

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
    capacity_gflops: Option<f64>,
    system: System,
    components: Components,
    battery_manager: Option<BatteryManager>,
    git_identity: Option<GitIdentity>,
}

// Git Identity Is Static Context, Read Once Rather Than Per Sample.
#[derive(Default)]
struct GitIdentity {
    name: Option<String>,
    email: Option<String>,
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
            capacity_gflops: None,
            system: System::new_all(),
            components: Components::new_with_refreshed_list(),
            battery_manager: BatteryManager::new().ok(),
            git_identity: None,
        }
    }

    pub fn calibrate_cpu(&mut self, budget: std::time::Duration) -> Result<()> {
        self.capacity_gflops = Some(crate::host::cpu::calibrate(budget)?);
        Ok(())
    }

    pub fn cpu_load(&mut self) -> Result<crate::host::cpu::Load> {
        let capacity = self.capacity_gflops.ok_or_else(|| crate::Error::Probe("CPU capacity has not been calibrated".into()))?;
        self.system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
        let utilization = f64::from(self.system.global_cpu_usage()).clamp(0.0, 100.0);
        Ok(crate::host::cpu::Load::measured(capacity, utilization))
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
        let power = Self::power_profile_context();
        let first_cpu = self.system.cpus().first();

        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("probe_status".to_owned(), Value::from("ok")),
            ("cpu_load_percent".to_owned(), Value::from(f64::from(self.system.global_cpu_usage()))),
            ("memory_total_bytes".to_owned(), integer(total_memory)),
            ("memory_used_bytes".to_owned(), integer(used_memory)),
            ("memory_used_gib".to_owned(), Value::from(used_memory as f64 / 1_073_741_824.0)),
            ("memory_total_gib".to_owned(), Value::from(total_memory as f64 / 1_073_741_824.0)),
            ("system_uptime_s".to_owned(), Value::from(System::uptime() as f64)),
            ("temperature_sensor_count".to_owned(), Value::Integer(temperatures.len() as i64)),
            ("cpu_architecture".to_owned(), Value::from(std::env::consts::ARCH)),
            ("logical_cpu_count".to_owned(), Value::Integer(self.system.cpus().len() as i64)),
        ]);
        if let Some(capacity) = self.capacity_gflops {
            let load = crate::host::cpu::Load::measured(capacity, f64::from(self.system.global_cpu_usage()).clamp(0.0, 100.0));
            values.insert("cpu_capacity_gflops".into(), Value::from(capacity));
            values.insert("cpu_load_gflops".into(), Value::from(load.load_gflops));
        }
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
        let git = self.git_identity.get_or_insert_with(git_identity);
        maybe_insert(&mut values, "git_user_name", git.name.clone().map(Value::from));
        maybe_insert(&mut values, "git_user_email", git.email.clone().map(Value::from));
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

fn git_identity() -> GitIdentity {
    let mut command = dokime_domain::process::command("git");
    command.args(["config", "--get-regexp", r"^user\.(name|email)$"]);
    let Ok(output) = super::run_command_bounded(command, system_probe_timeout()) else {
        return GitIdentity::default();
    };
    if !output.status.success() {
        return GitIdentity::default();
    }
    parse_git_identity(&String::from_utf8_lossy(&output.stdout))
}

// The Last Value Of A Key Wins, As With `git config --get`.
fn parse_git_identity(text: &str) -> GitIdentity {
    let mut identity = GitIdentity::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once(' ') else {
            continue;
        };
        let value = value.trim().to_owned();
        let slot = match key {
            "user.name" => &mut identity.name,
            "user.email" => &mut identity.email,
            _ => continue,
        };
        let present = !value.is_empty();
        *slot = present.then_some(value);
    }
    identity
}

// A System Probe Is Bounded By The Collection It Serves.
fn system_probe_timeout() -> std::time::Duration {
    crate::host::system::POLICY.attempt_timeout
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

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn git_identity_takes_the_last_value_of_each_key() {
        let output = "user.name First
user.email first@example.invalid
user.name Second Name
";
        let identity = super::parse_git_identity(output);
        assert_eq!(identity.name.as_deref(), Some("Second Name"));
        assert_eq!(identity.email.as_deref(), Some("first@example.invalid"));
    }
}
