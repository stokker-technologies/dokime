/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// System Collector: Identity, CPU, Memory, Temperature, Battery and Power Plan Rows

/* === Modules === */

#[cfg(target_os = "windows")]
mod native {
    use super::PowerPlan;
    use crate::platform::windows::power;

pub(super) fn power_plan() -> PowerPlan {
    let Ok((guid, name)) = power::active_plan() else {
        return PowerPlan::default();
    };
    let mode = name.as_deref().map(power::mode);
    PowerPlan {
        mode: mode.map(str::to_owned),
        performance_bias: mode.map(power::performance_bias),
        name,
        guid: Some(guid),
    }
}
}

#[cfg(unix)]
mod native {
    use super::PowerPlan;

pub(super) fn power_plan() -> PowerPlan {
    PowerPlan::default()
}
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

// Third Party Imports
use starship_battery::units::electric_potential::volt;
use starship_battery::units::energy::watt_hour;
use starship_battery::units::power::watt;
use starship_battery::units::ratio::{percent, ratio};
use starship_battery::units::thermodynamic_temperature::degree_celsius;
use starship_battery::units::time::second;
use starship_battery::{Battery, Manager};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind};

// Local Imports
use crate::Host;
use crate::collectors::cpu::Load;
use crate::domain::readings::System;
use crate::platform::{Lease, command};
use crate::settings;
use dokime::operations::Context;
use dokime::{Error, Result};

/* === Constants === */

const BYTES_PER_GIBIBYTE: f64 = 1_073_741_824.0;
const PERCENT_SCALE: f64 = 100.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// System Collection State, Created by the First Collection and Released at the Collection's Stop.
#[derive(Default)]
pub struct Collector {
    state: Arc<Mutex<Option<Listening>>>,
    git: Mutex<Option<GitIdentity>>,
}

/// Git Identity Is Static Context, Read Once Rather Than per Sample.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitIdentity {
    pub name: Option<String>,
    pub email: Option<String>,
}

struct Listening {
    state: Option<State>,
    _lease: Lease,
}

struct State {
    sample_index: i64,
    system: sysinfo::System,
    components: Components,
    batteries: Option<Manager>,
}

#[derive(Default)]
struct PowerPlan {
    name: Option<String>,
    guid: Option<String>,
    mode: Option<String>,
    performance_bias: Option<i64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Collector {
    /// Acquire Collection Resources Before Its First Tick.
    pub fn start(&self) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if state.is_none() {
            *state = Some(Listening { state: None, _lease: Lease::acquire()? });
        }
        Ok(())
    }

    /// Collect One System Row; a Failed Probe Becomes a Row Naming the Failure.
    pub async fn collect(&self, ctx: &Context) -> Result<System> {
        let timeout = settings::duration_value(ctx, settings::SYSTEM_PROBE_TIMEOUT).await?;
        let known = self.git.lock().unwrap_or_else(PoisonError::into_inner).clone();
        let git = match known {
            Some(git) => git,
            None => {
                let git = git_identity(timeout).await;
                *self.git.lock().unwrap_or_else(PoisonError::into_inner) = Some(git.clone());
                git
            }
        };
        let capacity_gflops = ctx.plugin::<Host>()?.cpu.capacity_gflops(ctx).await.ok();
        let state = self.state.clone();
        let collected = tokio::task::spawn_blocking(move || {
            let mut state = state.lock().unwrap_or_else(PoisonError::into_inner);
            let listening = state.as_mut().ok_or_else(|| Error::Invalid("Collect System Telemetry: Start the Collection Before Reading".into()))?;
            if listening.state.is_none() {
                listening.state = Some(State::new()?);
            }
            listening.state.as_mut().expect("State Created Above").collect(capacity_gflops, git)
        })
        .await
        .map_err(|error| Error::Invalid(format!("Collect System Telemetry: {error}; Restart the Plugin")))?;
        Ok(collected.unwrap_or_else(|error| System {
            probe_status: Some("error".into()),
            probe_error: Some(error.to_string()),
            ..System::default()
        }))
    }
}

impl Collector {
    /// Release the Native Inventory and the Git Identity When the Last Listener Leaves.
    pub fn stop(&self) {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).take();
        self.git.lock().unwrap_or_else(PoisonError::into_inner).take();
    }
}

impl State {
    fn new() -> Result<Self> {
        let invalid = |error: sysinfo::Error| Error::Invalid(format!("Read System Inventory: {error}; Restart the Plugin"));
        Ok(Self {
            sample_index: 0,
            system: sysinfo::System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything())).map_err(invalid)?,
            components: Components::new_with_refreshed_list().map_err(invalid)?,
            batteries: Manager::new().ok(),
        })
    }

    fn collect(&mut self, capacity_gflops: Option<f64>, git: GitIdentity) -> Result<System> {
        self.sample_index += 1;
        // Utilization Covers the Time Since the Previous Collection; the First One Reads Zero.
        self.system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
        let utilization_percent = f64::from(self.system.global_cpu_usage()).clamp(0.0, PERCENT_SCALE);
        let cpu = capacity_gflops.map(|capacity_gflops| Load::measured(capacity_gflops, utilization_percent));
        self.system.refresh_memory_specifics(MemoryRefreshKind::everything());
        self.components.refresh(true);
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let temperatures = self
            .components
            .iter()
            .filter_map(|component| component.temperature())
            .filter(|temperature| temperature.is_finite())
            .map(f64::from)
            .collect::<Vec<_>>();
        let first_cpu = self.system.cpus().first();
        let text = |value: &str| Some(value.trim()).filter(|value| !value.is_empty()).map(str::to_owned);
        let power = native::power_plan();
        let uptime =
            sysinfo::System::uptime().map_err(|error| Error::Invalid(format!("Read System Uptime: {error}; Restart the Plugin")))?;
        let row = System {
            sample_index: Some(self.sample_index),
            probe_status: Some("ok".into()),
            host_name: sysinfo::System::host_name().ok(),
            user: env::var("USERNAME").or_else(|_| env::var("USER")).ok(),
            git_user_name: git.name,
            git_user_email: git.email,
            os_name: sysinfo::System::name().ok(),
            os_version: sysinfo::System::long_os_version().ok(),
            kernel_version: sysinfo::System::kernel_version().ok(),
            cpu_architecture: Some(env::consts::ARCH.into()),
            cpu_name: first_cpu.and_then(|cpu| text(cpu.name())),
            cpu_brand: first_cpu.and_then(|cpu| text(cpu.brand())),
            cpu_vendor_id: first_cpu.and_then(|cpu| text(cpu.vendor_id())),
            cpu_frequency_mhz: first_cpu.map(|cpu| cpu.frequency() as i64),
            physical_cpu_count: sysinfo::System::physical_core_count().ok().map(|count| count as i64),
            logical_cpu_count: Some(self.system.cpus().len() as i64),
            cpu_load_percent: Some(utilization_percent),
            cpu_capacity_gflops: cpu.as_ref().map(|load| load.capacity_gflops),
            cpu_load_gflops: cpu.as_ref().map(|load| load.load_gflops),
            memory_total_bytes: Some(integer(total_memory)),
            memory_used_bytes: Some(integer(used_memory)),
            memory_pressure_percent: (total_memory > 0).then(|| used_memory as f64 / total_memory as f64 * PERCENT_SCALE),
            memory_used_gib: Some(used_memory as f64 / BYTES_PER_GIBIBYTE),
            memory_total_gib: Some(total_memory as f64 / BYTES_PER_GIBIBYTE),
            system_uptime_s: Some(uptime as f64),
            temperature_sensor_count: Some(temperatures.len() as i64),
            max_temperature_c: temperatures.into_iter().reduce(f64::max),
            power_plan_name: power.name,
            power_plan_guid: power.guid,
            power_mode: power.mode,
            power_mode_performance_bias: power.performance_bias,
            ..System::default()
        };
        let battery = self.batteries.as_ref().and_then(|manager| manager.batteries().ok()?.next()?.ok());
        Ok(match battery {
            Some(battery) => with_battery(row, &battery),
            None => row,
        })
    }
}

/* === Public Functions === */

/// The Last Value of Each Key Wins, as With `git config --get`.
pub fn parse_git_identity(text: &str) -> GitIdentity {
    let mut identity = GitIdentity::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once(' ') else {
            continue;
        };
        let slot = match key {
            "user.name" => &mut identity.name,
            "user.email" => &mut identity.email,
            _ => continue,
        };
        let value = value.trim();
        *slot = (!value.is_empty()).then(|| value.to_owned());
    }
    identity
}

/* === Private Functions === */

async fn git_identity(timeout: Duration) -> GitIdentity {
    let mut command = command::command("git");
    command.args(["config", "--get-regexp", r"^user\.(name|email)$"]);
    let output = tokio::task::spawn_blocking(move || command::bounded_output(command, timeout)).await;
    match output {
        Ok(Ok(output)) if output.status.success() => parse_git_identity(&String::from_utf8_lossy(&output.stdout)),
        _ => GitIdentity::default(),
    }
}

fn with_battery(row: System, battery: &Battery) -> System {
    let state = battery.state();
    let number = |value: f32| value.is_finite().then(|| f64::from(value));
    System {
        battery_percent: number(battery.state_of_charge().get::<percent>()),
        battery_state: Some(state.to_string()),
        battery_power_plugged: Some(matches!(state, starship_battery::State::Charging | starship_battery::State::Full)),
        battery_temperature_c: battery.temperature().and_then(|temperature| number(temperature.get::<degree_celsius>())),
        battery_vendor: battery.vendor().map(str::to_owned),
        battery_model: battery.model().map(str::to_owned),
        battery_serial_number: battery.serial_number().map(str::to_owned),
        battery_technology: Some(battery.technology().to_string()),
        battery_state_of_charge_ratio: number(battery.state_of_charge().get::<ratio>()),
        battery_energy_wh: number(battery.energy().get::<watt_hour>()),
        battery_energy_full_wh: number(battery.energy_full().get::<watt_hour>()),
        battery_energy_full_design_wh: number(battery.energy_full_design().get::<watt_hour>()),
        battery_energy_rate_w: number(battery.energy_rate().get::<watt>()),
        battery_voltage_v: number(battery.voltage().get::<volt>()),
        battery_state_of_health_ratio: number(battery.state_of_health().get::<ratio>()),
        battery_time_to_full_s: battery.time_to_full().and_then(|time| number(time.get::<second>())),
        battery_time_to_empty_s: battery.time_to_empty().and_then(|time| number(time.get::<second>())),
        battery_cycle_count: battery.cycle_count().map(i64::from),
        ..row
    }
}

fn integer(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn git_identity_takes_the_last_value_of_each_key() {
        let output = "user.name First\nuser.email first@example.invalid\nuser.name Second Name\nuser.email  \n";
        let identity = super::parse_git_identity(output);
        assert_eq!(identity.name.as_deref(), Some("Second Name"));
        assert_eq!(identity.email, None);
    }
}

#[cfg(all(test, target_os = "windows"))]
mod lifetime_tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{Collector, Listening};
    use crate::platform::windows::apartment;

    #[test]
    fn stop_releases_the_collection_lease_once() {
        let releases = Arc::new(AtomicUsize::new(0));
        let collector = Collector::default();
        *collector.state.lock().unwrap() = Some(Listening { state: None, _lease: apartment::tests::fake(releases.clone()) });
        assert_eq!(releases.load(Ordering::SeqCst), 0);
        collector.stop();
        assert_eq!(releases.load(Ordering::SeqCst), 1);
        collector.stop();
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }
}
