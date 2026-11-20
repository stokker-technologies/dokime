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

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

// Third Party Imports
use starship_battery::units::electric_potential::volt;
use starship_battery::units::energy::watt_hour;
use starship_battery::units::power::watt;
use starship_battery::units::ratio::{percent, ratio};
use starship_battery::units::thermodynamic_temperature::degree_celsius;
use starship_battery::units::time::second;
use starship_battery::{Battery, Manager, State as Charging};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::OnceCell;

// Local Imports
use crate::Host;
use crate::collectors::cpu::Load;
use crate::domain::telemetry::System;
use crate::settings;
use dokime::operations::Context;
use dokime::{Error, Result};

/* === Constants === */

const BYTES_PER_GIBIBYTE: f64 = 1_073_741_824.0;
const PERCENT_SCALE: f64 = 100.0;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// System Collection State Retained Between Collections, Created on the First One.
#[derive(Default)]
pub struct Collector {
    state: Arc<Mutex<Option<State>>>,
    git: OnceCell<GitIdentity>,
}

/// Git Identity Is Static Context, Read Once Rather Than per Sample.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitIdentity {
    pub name: Option<String>,
    pub email: Option<String>,
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
    /// Collect One System Row; a Failed Probe Becomes a Row Naming the Failure.
    pub async fn collect(&self, ctx: &Context) -> Result<System> {
        let timeout = settings::duration_value(ctx, settings::SYSTEM_PROBE_TIMEOUT).await?;
        let git = self.git.get_or_init(|| async { git_identity(timeout).await }).await.clone();
        let capacity_gflops = ctx.plugin::<Host>()?.cpu.capacity_gflops(ctx).await.ok();
        let state = self.state.clone();
        let collected = tokio::task::spawn_blocking(move || {
            let mut state = state.lock().unwrap_or_else(PoisonError::into_inner);
            if state.is_none() {
                *state = Some(State::new()?);
            }
            let state = state.as_mut().expect("State Created Above");
            state.collect(capacity_gflops, git)
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

impl State {
    fn new() -> Result<Self> {
        // Reading Components Enters the Apartment on This Thread for WMI and Never Leaves It.
        #[cfg(windows)]
        crate::platform::windows::apartment::retain()?;
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
        let power = power_plan();
        let uptime = sysinfo::System::uptime().map_err(|error| Error::Invalid(format!("Read System Uptime: {error}; Restart the Plugin")))?;
        let row = System {
            sample_index: Some(self.sample_index),
            probe_status: Some("ok".into()),
            host_name: sysinfo::System::host_name().ok(),
            user: std::env::var("USERNAME").or_else(|_| std::env::var("USER")).ok(),
            git_user_name: git.name,
            git_user_email: git.email,
            os_name: sysinfo::System::name().ok(),
            os_version: sysinfo::System::long_os_version().ok(),
            kernel_version: sysinfo::System::kernel_version().ok(),
            cpu_architecture: Some(std::env::consts::ARCH.into()),
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

/// Run a Command With Null Input, Capturing Its Output Within the Timeout and Killing It After.
///
/// Collectors call this from blocking tasks; a joined thread with its own runtime never nests runtimes.
pub fn bounded_output(command: Command, timeout: Duration) -> std::io::Result<Output> {
    std::thread::Builder::new()
        .name("host-probe".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
            runtime.block_on(async move {
                let mut command = tokio::process::Command::from(command);
                command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
                #[cfg(windows)]
                command.creation_flags(CREATE_NO_WINDOW);
                let mut child = command.spawn()?;
                let stdout = child.stdout.take().expect("Standard Output Is Piped");
                let stderr = child.stderr.take().expect("Standard Error Is Piped");
                let result = capture(stdout, stderr, child.wait(), timeout).await;
                if result.is_err() {
                    // Killing Also Waits for the Direct Child; the Cancelled Readers Strand No Threads.
                    let _ = child.kill().await;
                }
                result
            })
        })?
        .join()
        .map_err(|_| std::io::Error::other("Host Probe Worker Panicked"))?
}

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
    let mut command = Command::new("git");
    command.args(["config", "--get-regexp", r"^user\.(name|email)$"]);
    let output = tokio::task::spawn_blocking(move || bounded_output(command, timeout)).await;
    match output {
        Ok(Ok(output)) if output.status.success() => parse_git_identity(&String::from_utf8_lossy(&output.stdout)),
        _ => GitIdentity::default(),
    }
}

async fn capture(
    mut stdout: impl AsyncRead + Unpin, mut stderr: impl AsyncRead + Unpin,
    wait: impl std::future::Future<Output = std::io::Result<std::process::ExitStatus>>, timeout: Duration,
) -> std::io::Result<Output> {
    let capture = async {
        let mut stdout_bytes = Vec::new();
        let mut stderr_bytes = Vec::new();
        let (_, _, status) = tokio::try_join!(stdout.read_to_end(&mut stdout_bytes), stderr.read_to_end(&mut stderr_bytes), wait)?;
        Ok(Output {
            status,
            stdout: stdout_bytes,
            stderr: stderr_bytes,
        })
    };
    tokio::time::timeout(timeout, capture).await.unwrap_or_else(|_| Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Command Timed Out")))
}

#[cfg(windows)]
fn power_plan() -> PowerPlan {
    use crate::platform::windows::power;
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

/// Unix System Rows Carry No Power Plan; the Power Area Reads the ACPI Platform Profile.
#[cfg(unix)]
fn power_plan() -> PowerPlan {
    PowerPlan::default()
}

fn with_battery(row: System, battery: &Battery) -> System {
    let state = battery.state();
    let number = |value: f32| value.is_finite().then(|| f64::from(value));
    System {
        battery_percent: number(battery.state_of_charge().get::<percent>()),
        battery_state: Some(state.to_string()),
        battery_power_plugged: Some(matches!(state, Charging::Charging | Charging::Full)),
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
