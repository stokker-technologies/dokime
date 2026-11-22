/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Settings: Collection Intervals, Probe Selection and Timing, Sampling and CPU Load Control

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::operations::Context;
use dokime::{Error, OperationPolicy, Result, SettingDeclaration, SettingValue, setting_value};

/* === Constants === */

pub const SYSTEM_INTERVAL: &str = "system_interval";
pub const NETWORK_INTERVAL: &str = "network_interval";
pub const WIFI_LINK_INTERVAL: &str = "wifi_link_interval";
pub const WIFI_SCAN_INTERVAL: &str = "wifi_scan_interval";
pub const WIFI_LATENCY_INTERVAL: &str = "wifi_latency_interval";
pub const BLUETOOTH_INTERVAL: &str = "bluetooth_interval";

pub const WIFI_ADAPTER: &str = "wifi_adapter";
pub const BLUETOOTH_ADAPTER: &str = "bluetooth_adapter";
pub const LATENCY_HOST: &str = "latency_host";
pub const MINIMUM_SCAN_INTERVAL: &str = "minimum_scan_interval";

pub const NETWORK_PROBE_TIMEOUT: &str = "network_probe_timeout";
pub const SYSTEM_PROBE_TIMEOUT: &str = "system_probe_timeout";
pub const CPU_CALIBRATION_WINDOW: &str = "cpu_calibration_window";
pub const WIFI_LINK_PROBE_TIMEOUT: &str = "wifi_link_probe_timeout";
pub const WIFI_SCAN_PROBE_TIMEOUT: &str = "wifi_scan_probe_timeout";
pub const WIFI_SCAN_WINDOW: &str = "wifi_scan_window";
pub const LATENCY_PROCESS_TIMEOUT: &str = "latency_process_timeout";
pub const ECHO_REPLY_TIMEOUT: &str = "echo_reply_timeout";
pub const BLUETOOTH_PROBE_TIMEOUT: &str = "bluetooth_probe_timeout";
pub const BLUETOOTH_DEEP_REFRESH_INTERVAL: &str = "bluetooth_deep_refresh_interval";
pub const CONTROL_PROBE_TIMEOUT: &str = "control_probe_timeout";
pub const RADIO_SETTLE_TIMEOUT: &str = "radio_settle_timeout";

pub const MINIMUM_RATE_WINDOW: &str = "minimum_rate_window";
pub const CONGESTION_NETWORK_WEIGHT: &str = "congestion_network_weight";
pub const ECHO_PAYLOAD_BYTES: &str = "echo_payload_bytes";
pub const CALIBRATION_BATCH_ITERATIONS: &str = "calibration_batch_iterations";

pub const CPU_LOAD_PERIOD: &str = "cpu_load_period";
pub const CPU_LOAD_SAMPLE_PERIOD: &str = "cpu_load_sample_period";
pub const CPU_LOAD_WORKER_SLICE: &str = "cpu_load_worker_slice";
pub const CPU_LOAD_PROPORTIONAL_GAIN: &str = "cpu_load_proportional_gain";
pub const CPU_LOAD_INTEGRAL_GAIN: &str = "cpu_load_integral_gain";
pub const CPU_LOAD_INTEGRATOR_MINIMUM: &str = "cpu_load_integrator_minimum";
pub const CPU_LOAD_INTEGRATOR_MAXIMUM: &str = "cpu_load_integrator_maximum";
pub const CPU_LOAD_WORKER_SETTLEMENT: &str = "cpu_load_worker_settlement";
pub const CPU_LOAD_WORKER_BATCH_ITERATIONS: &str = "cpu_load_worker_batch_iterations";

/// CPU Load Holds Run Until Cancelled; Worker Settlement Must Fit This Cancellation Grace.
pub const CPU_LOAD_CANCELLATION_GRACE: Duration = Duration::from_secs(60);

/// One Mutating Control Reads, Writes and Verifies Within This Budget.
pub const CONTROL_BUDGET: Duration = Duration::from_secs(30);

/// A Restore May Write Several Outstanding Inverses, So It Has Twice the Control Budget.
pub const RESTORE_BUDGET: Duration = Duration::from_secs(60);

const DAY_MS: u64 = 86_400_000;

#[cfg(target_os = "windows")]
const ECHO_PAYLOAD_DEFAULT: i64 = 32;
#[cfg(not(target_os = "windows"))]
const ECHO_PAYLOAD_DEFAULT: i64 = 56;

/// Every Host Setting With Its Default and Accepted Range.
pub const SETTINGS: &[Setting] = &[
    Setting::milliseconds(
        SYSTEM_INTERVAL,
        "System Interval",
        "Time between system CPU, memory, battery and identity collections.",
        1_000,
        1,
        DAY_MS,
    ),
    Setting::milliseconds(
        NETWORK_INTERVAL,
        "Network Interval",
        "Time between adapter throughput and network notification collections.",
        1_000,
        1,
        DAY_MS,
    ),
    Setting::milliseconds(
        WIFI_LINK_INTERVAL,
        "Wi-Fi Link Interval",
        "Time between Wi-Fi link and throughput collections.",
        1_000,
        1,
        DAY_MS,
    ),
    Setting::milliseconds(
        WIFI_SCAN_INTERVAL,
        "Wi-Fi Scan Interval",
        "Time between Wi-Fi scan collections; active scans also honor Minimum Scan Interval.",
        60_000,
        1,
        DAY_MS,
    ),
    Setting::milliseconds(WIFI_LATENCY_INTERVAL, "Wi-Fi Latency Interval", "Time between ICMP latency collections.", 10_000, 1, DAY_MS),
    Setting::milliseconds(
        BLUETOOTH_INTERVAL,
        "Bluetooth Interval",
        "Time between Bluetooth collections; native inventories also honor Bluetooth Deep Refresh Interval.",
        5_000,
        1,
        DAY_MS,
    ),
    Setting::text(WIFI_ADAPTER, "Wi-Fi Adapter", "Adapter name to collect; empty selects an available adapter.", ""),
    Setting::text(BLUETOOTH_ADAPTER, "Bluetooth Adapter", "Adapter name to collect; empty selects an available adapter.", ""),
    Setting::text(LATENCY_HOST, "Latency Host", "IP address or hostname receiving one ICMP echo per latency collection.", "1.1.1.1"),
    Setting::milliseconds(
        MINIMUM_SCAN_INTERVAL,
        "Minimum Scan Interval",
        "Minimum time between active Wi-Fi scans; intervening collections use the driver's cache.",
        30_000,
        1_000,
        3_600_000,
    ),
    Setting::milliseconds(
        NETWORK_PROBE_TIMEOUT,
        "Network Probe Timeout",
        "Adapter inventory retry budget and maximum wait for Unix connectivity and inventory commands.",
        2_000,
        1,
        2_000,
    ),
    Setting::milliseconds(SYSTEM_PROBE_TIMEOUT, "System Probe Timeout", "Maximum wait for the Git identity subprocess.", 2_000, 1, 2_000),
    Setting::milliseconds(
        CPU_CALIBRATION_WINDOW,
        "CPU Calibration Window",
        "Startup window for the CPU capacity calibration kernel.",
        1_000,
        1,
        1_000,
    ),
    Setting::milliseconds(
        WIFI_LINK_PROBE_TIMEOUT,
        "Wi-Fi Link Probe Timeout",
        "Maximum wait for each native link command on Unix.",
        2_000,
        1,
        2_000,
    ),
    Setting::milliseconds(
        WIFI_SCAN_PROBE_TIMEOUT,
        "Wi-Fi Scan Probe Timeout",
        "Maximum wait for each NetworkManager scan command on Unix.",
        12_000,
        1,
        12_000,
    ),
    Setting::milliseconds(
        WIFI_SCAN_WINDOW,
        "Wi-Fi Scan Window",
        "Maximum wait for an active Windows WLAN scan notification before reading the BSS list.",
        6_000,
        1,
        6_000,
    ),
    Setting::milliseconds(LATENCY_PROCESS_TIMEOUT, "Latency Process Timeout", "Maximum wait for the Unix ping process.", 4_000, 1, 4_000),
    Setting::milliseconds(
        ECHO_REPLY_TIMEOUT,
        "Echo Reply Timeout",
        "Maximum wait for one ICMP reply; Unix ping rounds this up to whole seconds.",
        2_000,
        1,
        2_000,
    ),
    Setting::milliseconds(
        BLUETOOTH_PROBE_TIMEOUT,
        "Bluetooth Probe Timeout",
        "Maximum wait for each BlueZ D-Bus method on Unix; Windows uses synchronous native inventory.",
        6_000,
        1,
        6_000,
    ),
    Setting::milliseconds(
        BLUETOOTH_DEEP_REFRESH_INTERVAL,
        "Bluetooth Deep Refresh Interval",
        "Time between native Bluetooth inventories; collections between refreshes reuse the cached snapshot.",
        300_000,
        1,
        DAY_MS,
    ),
    Setting::milliseconds(
        CONTROL_PROBE_TIMEOUT,
        "Control Probe Timeout",
        "Maximum wait for each radio control D-Bus method on Unix, including restoration.",
        30_000,
        1,
        30_000,
    ),
    Setting::milliseconds(
        RADIO_SETTLE_TIMEOUT,
        "Radio Settle Timeout",
        "Windows radio state notification wait during a write or inverse restoration.",
        15_000,
        1,
        15_000,
    ),
    Setting::milliseconds(
        MINIMUM_RATE_WINDOW,
        "Minimum Rate Window",
        "Minimum elapsed denominator for network and Wi-Fi throughput calculations.",
        1,
        1,
        60_000,
    ),
    Setting::number(
        CONGESTION_NETWORK_WEIGHT,
        "Congestion Weight [%/network]",
        "Contribution of one fully overlapping Wi-Fi network to the congestion score, capped at 100 percent.",
        10.0,
        Some(0.01),
        Some(100.0),
    ),
    Setting::integer(
        ECHO_PAYLOAD_BYTES,
        "Echo Payload [bytes]",
        "ICMP echo payload size; defaults match native ping on each platform.",
        ECHO_PAYLOAD_DEFAULT,
        1,
        65_000,
    ),
    Setting::integer(
        CALIBRATION_BATCH_ITERATIONS,
        "Calibration Batch [iterations]",
        "CPU calibration kernel iterations between deadline checks.",
        4_096,
        1,
        65_536,
    ),
    Setting::milliseconds(
        CPU_LOAD_PERIOD,
        "Controller Period",
        "Time between CPU load controller updates and control stream rows.",
        250,
        1,
        60_000,
    ),
    Setting::milliseconds(
        CPU_LOAD_SAMPLE_PERIOD,
        "CPU Sampling Period",
        "Time between shared CPU utilization samples; must not exceed the controller period.",
        250,
        1,
        60_000,
    ),
    Setting::milliseconds(
        CPU_LOAD_WORKER_SLICE,
        "Worker Slice",
        "Duty-cycle slice of each load worker; must not exceed the controller period.",
        10,
        1,
        1_000,
    ),
    Setting::number(
        CPU_LOAD_PROPORTIONAL_GAIN,
        "Proportional Gain",
        "Controller duty change per unit of normalized load error.",
        0.5,
        Some(0.0),
        None,
    ),
    Setting::number(
        CPU_LOAD_INTEGRAL_GAIN,
        "Integral Gain [1/s]",
        "Controller integral gain on normalized load error.",
        0.5,
        Some(0.0),
        None,
    ),
    Setting::number(
        CPU_LOAD_INTEGRATOR_MINIMUM,
        "Integrator Minimum",
        "Lower anti-windup bound of the controller integrator; must not exceed zero.",
        -1.0,
        None,
        Some(0.0),
    ),
    Setting::number(
        CPU_LOAD_INTEGRATOR_MAXIMUM,
        "Integrator Maximum",
        "Upper anti-windup bound of the controller integrator; must not be below zero.",
        1.0,
        Some(0.0),
        None,
    ),
    Setting::milliseconds(
        CPU_LOAD_WORKER_SETTLEMENT,
        "Worker Settlement Budget",
        "Maximum wait for load workers to stop after cancellation; must cover one worker slice.",
        2_000,
        1,
        60_000,
    ),
    Setting::integer(
        CPU_LOAD_WORKER_BATCH_ITERATIONS,
        "Worker Batch [iterations]",
        "Load worker kernel iterations between stop checks.",
        4_096,
        1,
        65_536,
    ),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Flat Setting Declaration With the Range Its Saved Value Must Respect.
pub struct Setting {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub kind: Kind,
}

/// The CPU Load Controller and Worker Parameters, Validated Together.
#[derive(Clone, Debug, PartialEq)]
pub struct CpuLoad {
    pub period: Duration,
    pub sample_period: Duration,
    pub slice: Duration,
    pub proportional_gain: f64,
    pub integral_gain: f64,
    pub integral_min: f64,
    pub integral_max: f64,
    pub settlement: Duration,
    pub batch_iterations: u32,
}

/* === Enums === */

pub enum Kind {
    Milliseconds { default: u64, min: u64, max: u64 },
    Integer { default: i64, min: i64, max: i64 },
    Number { default: f64, min: Option<f64>, max: Option<f64> },
    Text { default: &'static str },
}

/* === Traits === */

/* === Implementations === */

impl Setting {
    const fn milliseconds(name: &'static str, title: &'static str, description: &'static str, default: u64, min: u64, max: u64) -> Self {
        Self {
            name,
            title,
            description,
            kind: Kind::Milliseconds { default, min, max },
        }
    }
    const fn integer(name: &'static str, title: &'static str, description: &'static str, default: i64, min: i64, max: i64) -> Self {
        Self {
            name,
            title,
            description,
            kind: Kind::Integer { default, min, max },
        }
    }
    const fn number(
        name: &'static str, title: &'static str, description: &'static str, default: f64, min: Option<f64>, max: Option<f64>,
    ) -> Self {
        Self {
            name,
            title,
            description,
            kind: Kind::Number { default, min, max },
        }
    }
    const fn text(name: &'static str, title: &'static str, description: &'static str, default: &'static str) -> Self {
        Self {
            name,
            title,
            description,
            kind: Kind::Text { default },
        }
    }

    pub fn declaration(&self) -> SettingDeclaration {
        let (minimum, maximum) = match self.kind {
            Kind::Milliseconds { min, max, .. } => (
                Some(setting_value::Value::Duration(duration(Duration::from_millis(min)))),
                Some(setting_value::Value::Duration(duration(Duration::from_millis(max)))),
            ),
            Kind::Integer { min, max, .. } => (Some(setting_value::Value::Integer(min)), Some(setting_value::Value::Integer(max))),
            Kind::Number { min, max, .. } => (min.map(setting_value::Value::Number), max.map(setting_value::Value::Number)),
            Kind::Text { .. } => (None, None),
        };
        SettingDeclaration {
            name: self.name.into(),
            title: self.title.into(),
            description: self.description.into(),
            default: Some(self.default_value()),
            minimum: minimum.map(|value| SettingValue { value: Some(value) }),
            maximum: maximum.map(|value| SettingValue { value: Some(value) }),
        }
    }

    pub fn default_value(&self) -> SettingValue {
        let value = match self.kind {
            Kind::Milliseconds { default, .. } => setting_value::Value::Duration(duration(Duration::from_millis(default))),
            Kind::Integer { default, .. } => setting_value::Value::Integer(default),
            Kind::Number { default, .. } => setting_value::Value::Number(default),
            Kind::Text { default } => setting_value::Value::Text(default.into()),
        };
        SettingValue { value: Some(value) }
    }
}

impl CpuLoad {
    pub async fn read(ctx: &Context) -> Result<Self> {
        let load = Self {
            period: ctx.duration_setting(CPU_LOAD_PERIOD).await?,
            sample_period: ctx.duration_setting(CPU_LOAD_SAMPLE_PERIOD).await?,
            slice: ctx.duration_setting(CPU_LOAD_WORKER_SLICE).await?,
            proportional_gain: number(ctx, CPU_LOAD_PROPORTIONAL_GAIN).await?,
            integral_gain: number(ctx, CPU_LOAD_INTEGRAL_GAIN).await?,
            integral_min: number(ctx, CPU_LOAD_INTEGRATOR_MINIMUM).await?,
            integral_max: number(ctx, CPU_LOAD_INTEGRATOR_MAXIMUM).await?,
            settlement: ctx.duration_setting(CPU_LOAD_WORKER_SETTLEMENT).await?,
            batch_iterations: u32::try_from(integer(ctx, CPU_LOAD_WORKER_BATCH_ITERATIONS).await?)
                .map_err(|_| Error::Invalid("Read CPU Load Settings: Configure a Smaller Worker Batch".into()))?,
        };
        load.validate()?;
        Ok(load)
    }

    /// Relations Between the CPU Load Settings That Single-Setting Ranges Cannot Express.
    pub fn validate(&self) -> Result<()> {
        if self.settlement < self.slice || self.sample_period > self.period || self.slice > self.period {
            return Err(Error::Invalid(
                "Read CPU Load Settings: Fit Sampling and Slices in a Controller Period and Let Settlement Cover a \
                 Slice Within the Cancellation Grace"
                    .into(),
            ));
        }
        if self.integral_min >= self.integral_max {
            return Err(Error::Invalid("Read CPU Load Settings: Keep the Integrator Minimum Below Its Maximum".into()));
        }

        Ok(())
    }
}

/* === Public Functions === */

/// A Single-Attempt Operation Policy: the Whole Budget for One Attempt, Then the Cancellation Grace.
pub fn policy(budget: Duration, grace: Duration) -> OperationPolicy {
    let duration = |value: Duration| prost_types::Duration::try_from(value).ok();
    OperationPolicy {
        timeout: duration(budget),
        attempt_timeout: duration(budget),
        cancellation_timeout: duration(grace),
        retries: None,
    }
}

pub fn catalog() -> Vec<SettingDeclaration> {
    SETTINGS.iter().map(Setting::declaration).collect()
}

/// Validate Relations Between Settings When the Plugin Connects.
pub async fn validate(ctx: &Context) -> Result<()> {
    CpuLoad::read(ctx).await?;
    Ok(())
}

pub async fn integer(ctx: &Context, name: &str) -> Result<i64> {
    match ctx.setting(name).await?.value {
        Some(setting_value::Value::Integer(value)) => Ok(value),
        _ => Err(Error::Invalid(format!("Read Setting {name}: Configure an Integer in the Plugin Settings"))),
    }
}

pub async fn number(ctx: &Context, name: &str) -> Result<f64> {
    match ctx.setting(name).await?.value {
        Some(setting_value::Value::Number(value)) => Ok(value),
        _ => Err(Error::Invalid(format!("Read Setting {name}: Configure a Number in the Plugin Settings"))),
    }
}

pub async fn text(ctx: &Context, name: &str) -> Result<String> {
    match ctx.setting(name).await?.value {
        Some(setting_value::Value::Text(value)) => Ok(value),
        _ => Err(Error::Invalid(format!("Read Setting {name}: Configure a Text Value in the Plugin Settings"))),
    }
}

/// A Text Selection Where Empty Means Any Available Choice.
pub async fn selected(ctx: &Context, name: &str) -> Result<Option<String>> {
    let value = text(ctx, name).await?;
    let value = value.trim();
    Ok((!value.is_empty()).then(|| value.to_owned()))
}

pub fn setting(name: &str) -> Result<&'static Setting> {
    SETTINGS.iter().find(|setting| setting.name == name).ok_or_else(|| Error::Invalid(format!("Read Setting {name}: Declare It First")))
}

/* === Private Functions === */

fn duration(value: Duration) -> prost_types::Duration {
    prost_types::Duration {
        seconds: value.as_secs() as i64,
        nanos: value.subsec_nanos() as i32,
    }
}

/* === Tests === */
