/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Plugin Settings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};

/* === Constants === */

// Declared Observer Cadences And Plugin Setting Defaults Share These Values.
pub(crate) const SYSTEM_INTERVAL_MS: u64 = 1_000;
pub(crate) const NETWORK_INTERVAL_MS: u64 = 1_000;
pub(crate) const WIFI_LINK_INTERVAL_MS: u64 = 1_000;
pub(crate) const WIFI_SCAN_INTERVAL_MS: u64 = 60_000;
pub(crate) const WIFI_LATENCY_INTERVAL_MS: u64 = 10_000;
pub(crate) const BLUETOOTH_INTERVAL_MS: u64 = 5_000;

pub(crate) const LATENCY_HOST: &str = "1.1.1.1";

// Operation Policies Remain Static Harmos Contracts.
pub(crate) const SYSTEM_BUDGET_MS: u64 = 2_000;
pub(crate) const NETWORK_BUDGET_MS: u64 = 2_000;
pub(crate) const WIFI_LINK_BUDGET_MS: u64 = 2_000;
pub(crate) const WIFI_SCAN_BUDGET_MS: u64 = 12_000;
pub(crate) const WIFI_LATENCY_BUDGET_MS: u64 = 4_000;
pub(crate) const BLUETOOTH_BUDGET_MS: u64 = 6_000;
pub(crate) const CONTROL_BUDGET_MS: u64 = 30_000;
pub(crate) const RESTORE_BUDGET_MS: u64 = CONTROL_BUDGET_MS * 2;
// The Publication Window Bounds Replay And Native Notification Batches.
pub(crate) const STREAM_WINDOW: usize = 4_096;

// CPU Load Holds Run Until Cancelled; Runtime Settlement Must Fit The Cancellation Grace.
pub(crate) const CPU_LOAD_HOLD_BUDGET: Duration = Duration::from_secs(1 << 32);
pub(crate) const CPU_LOAD_CANCELLATION_GRACE: Duration = Duration::from_secs(60);

pub(crate) const CPU_LOAD_DEFAULTS: CpuLoadSettings = CpuLoadSettings {
    period_ms: 250,
    sample_ms: 250,
    slice_ms: 10,
    proportional_gain: 0.5,
    integral_gain: 0.5,
    integral_min: -1.0,
    integral_max: 1.0,
    settlement_ms: 2_000,
    batch_iterations: 4_096,
};

#[cfg(target_os = "windows")]
const ECHO_PAYLOAD_BYTES: u32 = 32;

#[cfg(not(target_os = "windows"))]
const ECHO_PAYLOAD_BYTES: u32 = 56;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::settings]
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub cadence: Cadence,
    pub probes: Probes,
    pub timing: Timing,
    pub sampling: Sampling,
    pub cpu_load: CpuLoadSettings,
}

#[dokime::section(
    label = "Collection Intervals",
    description = "Sampling cadence for each host stream. Changes apply on plugin restart or Retry."
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cadence {
    #[setting(
        label = "System Interval [ms]",
        default = SYSTEM_INTERVAL_MS,
        min = 1,
        max = 86_400_000,
        description = "Time between system CPU, memory, battery and identity collections while requested."
    )]
    pub system_interval_ms: u64,

    #[setting(
        label = "Network Interval [ms]",
        default = NETWORK_INTERVAL_MS,
        min = 1,
        max = 86_400_000,
        description = "Time between adapter throughput and network notification collections while requested."
    )]
    pub network_interval_ms: u64,

    #[setting(
        label = "Wi-Fi Link Interval [ms]",
        default = WIFI_LINK_INTERVAL_MS,
        min = 1,
        max = 86_400_000,
        description = "Time between Wi-Fi link and throughput collections while requested."
    )]
    pub wifi_link_interval_ms: u64,

    #[setting(
        label = "Wi-Fi Scan Interval [ms]",
        default = WIFI_SCAN_INTERVAL_MS,
        min = 1,
        max = 86_400_000,
        description = "Time between Wi-Fi scan collections; active scans also honor Minimum Scan Interval."
    )]
    pub wifi_scan_interval_ms: u64,

    #[setting(
        label = "Wi-Fi Latency Interval [ms]",
        default = WIFI_LATENCY_INTERVAL_MS,
        min = 1,
        max = 86_400_000,
        description = "Time between ICMP latency collections while requested."
    )]
    pub wifi_latency_interval_ms: u64,

    #[setting(
        label = "Bluetooth Interval [ms]",
        default = BLUETOOTH_INTERVAL_MS,
        min = 1,
        max = 86_400_000,
        description = "Time between Bluetooth collections; native inventories also honor Deep Refresh Interval."
    )]
    pub bluetooth_interval_ms: u64,
}

#[dokime::section(
    label = "Collection",
    description = "Adapter selection and active-scan protection. Changes apply on plugin restart or Retry."
)]
#[derive(Clone, Debug, PartialEq)]
pub struct Probes {
    #[setting(label = "Wi-Fi Adapter", description = "Adapter name to collect; empty selects an available adapter.")]
    pub wifi_adapter_name: Option<String>,

    #[setting(label = "Bluetooth Adapter", description = "Adapter name to collect; empty selects an available adapter.")]
    pub bluetooth_adapter_name: Option<String>,

    #[setting(label = "Latency Host", default = LATENCY_HOST, description = "IP address or hostname receiving one ICMP echo per latency collection.")]
    pub latency_host: String,

    #[setting(
        label = "Minimum Scan Interval [s]",
        default = 30,
        min = 1,
        max = 3_600,
        step = 1,
        description = "Minimum time between active Wi-Fi scans; intervening collections use the driver's cache."
    )]
    pub minimum_scan_interval_s: u32,
}

#[dokime::section(
    label = "Probe Timing",
    description = "Native probe bounds within the SDK's fixed operation deadlines. Changes apply on restart or Retry."
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timing {
    #[setting(label = "Network Probe Timeout [ms]", default = NETWORK_BUDGET_MS, min = 1, max = 2_000,
        description = "Adapter inventory retry budget and maximum wait for Unix connectivity and inventory commands.")]
    pub network_probe_timeout_ms: u64,

    #[setting(label = "System Probe Timeout [ms]", default = SYSTEM_BUDGET_MS, min = 1, max = 2_000,
        description = "Maximum wait for the Git identity subprocess.")]
    pub system_probe_timeout_ms: u64,

    #[setting(label = "CPU Calibration Window [ms]", default = SYSTEM_BUDGET_MS / 2, min = 1, max = 1_000,
        description = "Startup window for the existing CPU capacity calibration kernel.")]
    pub cpu_calibration_ms: u64,

    #[setting(label = "Wi-Fi Link Probe Timeout [ms]", default = WIFI_LINK_BUDGET_MS, min = 1, max = 2_000,
        description = "Maximum wait for each native link command on Unix.")]
    pub wifi_link_probe_timeout_ms: u64,

    #[setting(label = "Wi-Fi Scan Probe Timeout [ms]", default = WIFI_SCAN_BUDGET_MS, min = 1, max = 12_000,
        description = "Maximum wait for each NetworkManager scan command on Unix.")]
    pub wifi_scan_probe_timeout_ms: u64,

    #[setting(label = "Wi-Fi Scan Window [ms]", default = WIFI_SCAN_BUDGET_MS / 2, min = 1, max = 6_000,
        description = "Maximum wait for an active Windows WLAN scan notification before reading the BSS list.")]
    pub wifi_scan_window_ms: u64,

    #[setting(label = "Latency Process Timeout [ms]", default = WIFI_LATENCY_BUDGET_MS, min = 1, max = 4_000,
        description = "Maximum wait for the Unix ping process.")]
    pub wifi_latency_probe_timeout_ms: u64,

    #[setting(label = "Echo Reply Timeout [ms]", default = WIFI_LATENCY_BUDGET_MS / 2, min = 1, max = 2_000,
        description = "Maximum wait for one ICMP reply; Unix ping rounds this up to whole seconds.")]
    pub wifi_echo_timeout_ms: u64,

    #[setting(label = "Bluetooth Probe Timeout [ms]", default = BLUETOOTH_BUDGET_MS, min = 1, max = 6_000,
        description = "Maximum wait for each BlueZ D-Bus method on Unix; Windows uses synchronous native inventory.")]
    pub bluetooth_probe_timeout_ms: u64,

    #[setting(
        label = "Bluetooth Deep Refresh Interval [ms]",
        default = 300_000,
        min = 1,
        max = 86_400_000,
        description = "Time between native Bluetooth inventories; collections between refreshes reuse the cached snapshot."
    )]
    pub bluetooth_deep_refresh_ms: u64,

    #[setting(label = "Control Probe Timeout [ms]", default = CONTROL_BUDGET_MS, min = 1, max = 30_000,
        description = "Maximum wait for each radio control D-Bus method on Unix, including restoration.")]
    pub control_probe_timeout_ms: u64,

    #[setting(label = "Radio Settle Timeout [ms]", default = CONTROL_BUDGET_MS / 2, min = 1, max = 15_000,
        description = "Windows radio state notification wait during a write or inverse restoration.")]
    pub radio_settle_timeout_ms: u64,
}

#[dokime::section(label = "Sampling", description = "Measurement parameters. Changes apply on plugin restart or Retry.")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sampling {
    #[setting(
        label = "Minimum Rate Window [ms]",
        default = 1,
        min = 1,
        max = 60_000,
        description = "Minimum elapsed denominator for network and Wi-Fi throughput calculations."
    )]
    pub minimum_rate_window_ms: u64,

    #[setting(
        label = "Congestion Weight [%/network]",
        default = 10.0,
        min = 0.01,
        max = 100.0,
        description = "Contribution of one fully overlapping Wi-Fi network to the congestion score, capped at 100 percent."
    )]
    pub congestion_network_weight: f64,

    #[setting(label = "Echo Payload [bytes]", default = ECHO_PAYLOAD_BYTES, min = 1, max = 65_000,
        description = "ICMP echo payload size; defaults match native ping on each platform.")]
    pub echo_payload_bytes: u32,

    #[setting(
        label = "Calibration Batch [iterations]",
        default = 4_096,
        min = 1,
        max = 65_536,
        description = "Existing CPU calibration kernel iterations between deadline checks."
    )]
    pub calibration_batch_iterations: u32,
}

#[dokime::section(
    label = "CPU Load Control",
    description = "CPU load hold controller and worker parameters. Changes apply on plugin restart or Retry."
)]
#[derive(Clone, Debug, PartialEq)]
pub struct CpuLoadSettings {
    #[setting(label = "Controller Period [ms]", default = CPU_LOAD_DEFAULTS.period_ms, min = 1, max = 60_000,
        description = "Time between CPU load controller updates and control stream rows.")]
    pub period_ms: u32,

    #[setting(label = "CPU Sampling Period [ms]", default = CPU_LOAD_DEFAULTS.sample_ms, min = 1, max = 60_000,
        description = "Time between shared CPU utilization samples; must not exceed the controller period.")]
    pub sample_ms: u32,

    #[setting(label = "Worker Slice [ms]", default = CPU_LOAD_DEFAULTS.slice_ms, min = 1, max = 1_000,
        description = "Duty-cycle slice of each load worker; must not exceed the controller period.")]
    pub slice_ms: u32,

    #[setting(label = "Proportional Gain", default = CPU_LOAD_DEFAULTS.proportional_gain, min = 0.0,
        description = "Controller duty change per unit of normalized load error.")]
    pub proportional_gain: f64,

    #[setting(label = "Integral Gain [1/s]", default = CPU_LOAD_DEFAULTS.integral_gain, min = 0.0,
        description = "Controller integral gain on normalized load error.")]
    pub integral_gain: f64,

    #[setting(label = "Integrator Minimum", default = CPU_LOAD_DEFAULTS.integral_min,
        description = "Lower anti-windup bound of the controller integrator; must not exceed zero.")]
    pub integral_min: f64,

    #[setting(label = "Integrator Maximum", default = CPU_LOAD_DEFAULTS.integral_max,
        description = "Upper anti-windup bound of the controller integrator; must not be below zero.")]
    pub integral_max: f64,

    #[setting(label = "Worker Settlement Budget [ms]", default = CPU_LOAD_DEFAULTS.settlement_ms, min = 1, max = 60_000,
        description = "Maximum wait for load workers to stop after cancellation; must cover one worker slice.")]
    pub settlement_ms: u32,

    #[setting(label = "Worker Batch [iterations]", default = CPU_LOAD_DEFAULTS.batch_iterations, min = 1, max = 65_536,
        description = "Load worker kernel iterations between stop checks.")]
    pub batch_iterations: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostOptions {
    pub cadence: Cadence,
    pub wifi_adapter_name: Option<String>,
    pub bluetooth_adapter_name: Option<String>,
    pub latency_host: Option<String>,
    pub minimum_scan_interval: Duration,
    pub timing: Timing,
    pub sampling: Sampling,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for Settings {
    fn default() -> Self {
        dokime::plugin::Configuration::declared::<Self>().settings::<Self>().expect("host setting declarations have valid defaults")
    }
}

impl Default for CpuLoadSettings {
    fn default() -> Self {
        CPU_LOAD_DEFAULTS
    }
}

impl CpuLoadSettings {
    pub fn validate(&self) -> crate::Result<()> {
        if self.period_ms == 0
            || self.sample_ms == 0
            || self.slice_ms == 0
            || self.batch_iterations == 0
            || self.settlement_ms < self.slice_ms
            || self.settlement() > CPU_LOAD_CANCELLATION_GRACE
            || self.sample_ms > self.period_ms
            || self.slice_ms > self.period_ms
        {
            return Err(crate::Error::Probe(
                "CPU timing must be positive; sampling and slices must fit a control period; settlement must cover a slice and fit the cancellation grace".into(),
            ));
        }
        if !self.proportional_gain.is_finite()
            || self.proportional_gain < 0.0
            || !self.integral_gain.is_finite()
            || self.integral_gain < 0.0
            || !self.integral_min.is_finite()
            || !self.integral_max.is_finite()
            || self.integral_min > 0.0
            || self.integral_max < 0.0
            || self.integral_min >= self.integral_max
        {
            return Err(crate::Error::Probe("CPU gains must be finite and nonnegative; finite integrator bounds must bracket zero".into()));
        }
        Ok(())
    }

    pub fn period(&self) -> Duration {
        Duration::from_millis(u64::from(self.period_ms))
    }

    pub fn sample_period(&self) -> Duration {
        Duration::from_millis(u64::from(self.sample_ms))
    }

    pub fn slice(&self) -> Duration {
        Duration::from_millis(u64::from(self.slice_ms))
    }

    pub fn settlement(&self) -> Duration {
        Duration::from_millis(u64::from(self.settlement_ms))
    }
}

impl From<Settings> for HostOptions {
    fn from(settings: Settings) -> Self {
        Self {
            cadence: settings.cadence,
            wifi_adapter_name: settings.probes.wifi_adapter_name.and_then(selected),
            bluetooth_adapter_name: settings.probes.bluetooth_adapter_name.and_then(selected),
            latency_host: selected(settings.probes.latency_host),
            minimum_scan_interval: Duration::from_secs(u64::from(settings.probes.minimum_scan_interval_s)),
            timing: settings.timing,
            sampling: settings.sampling,
        }
    }
}

impl Default for HostOptions {
    fn default() -> Self {
        Self::from(Settings::default())
    }
}

/* === Public Functions === */

// The SDK Operation Declaration Requires A Const Policy, Including Retry And Cancellation Semantics.
pub(crate) const fn collection_policy(budget_ms: u64) -> OperationPolicy {
    let budget = Duration::from_millis(budget_ms);
    OperationPolicy {
        timeout: budget,
        attempt_timeout: budget,
        cancellation_timeout: Duration::ZERO,
        retries: RetryPolicy::Never,
    }
}

/* === Private Functions === */

fn selected(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_owned())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_defaults_resolve_to_existing_probe_behavior() {
        let configuration = dokime::plugin::Configuration::declared::<Settings>();
        let resolved = configuration.settings::<Settings>().unwrap();
        let options = HostOptions::from(resolved);
        assert_eq!(options.minimum_scan_interval, Duration::from_secs(30));
        assert_eq!(options.latency_host.as_deref(), Some("1.1.1.1"));
        assert!(options.wifi_adapter_name.is_none() && options.bluetooth_adapter_name.is_none());
        // Exercise Declaration Defaults And Resolution Together Against The Previous Probe Parameters.
        let timing = options.timing;
        assert_eq!(
            [
                timing.system_probe_timeout_ms,
                timing.network_probe_timeout_ms,
                timing.cpu_calibration_ms,
                timing.wifi_link_probe_timeout_ms,
                timing.wifi_scan_probe_timeout_ms,
                timing.wifi_scan_window_ms,
                timing.wifi_latency_probe_timeout_ms,
                timing.wifi_echo_timeout_ms,
                timing.bluetooth_probe_timeout_ms,
                timing.bluetooth_deep_refresh_ms,
                timing.control_probe_timeout_ms,
                timing.radio_settle_timeout_ms
            ],
            [2_000, 2_000, 1_000, 2_000, 12_000, 6_000, 4_000, 2_000, 6_000, 300_000, 30_000, 15_000]
        );
        let sampling = options.sampling;
        assert_eq!(
            (sampling.minimum_rate_window_ms, sampling.congestion_network_weight, sampling.calibration_batch_iterations),
            (1, 10.0, 4_096)
        );
        #[cfg(target_os = "windows")]
        assert_eq!(sampling.echo_payload_bytes, 32);
        #[cfg(not(target_os = "windows"))]
        assert_eq!(sampling.echo_payload_bytes, 56);
    }

    #[test]
    fn saved_probe_options_are_resolved_and_normalized() {
        let mut configuration = dokime::plugin::Configuration::declared::<Settings>();
        configuration.settings.insert("wifi_adapter_name".into(), dokime::Value::from("  wlan-test  "));
        configuration.settings.insert("bluetooth_adapter_name".into(), dokime::Value::from("   "));
        configuration.settings.insert("latency_host".into(), dokime::Value::from("  127.0.0.1  "));
        configuration.settings.insert("minimum_scan_interval_s".into(), dokime::Value::from(47_u32));
        configuration.settings.insert("wifi_echo_timeout_ms".into(), dokime::Value::from(175_u64));
        let options = HostOptions::from(configuration.settings::<Settings>().unwrap());
        assert_eq!(options.wifi_adapter_name.as_deref(), Some("wlan-test"));
        assert_eq!(options.bluetooth_adapter_name, None);
        assert_eq!(options.latency_host.as_deref(), Some("127.0.0.1"));
        assert_eq!(options.minimum_scan_interval, Duration::from_secs(47));
        assert_eq!(options.timing.wifi_echo_timeout_ms, 175);
    }

    #[test]
    fn cpu_load_settings_validate_timing_and_gains() {
        let defaults = dokime::plugin::Configuration::declared::<Settings>().settings::<Settings>().unwrap().cpu_load;
        assert_eq!(defaults, CPU_LOAD_DEFAULTS);
        assert!(defaults.validate().is_ok());
        for settings in [
            CpuLoadSettings { period_ms: 0, ..CPU_LOAD_DEFAULTS },
            CpuLoadSettings {
                sample_ms: CPU_LOAD_DEFAULTS.period_ms + 1,
                ..CPU_LOAD_DEFAULTS
            },
            CpuLoadSettings { settlement_ms: 1, ..CPU_LOAD_DEFAULTS },
            CpuLoadSettings {
                settlement_ms: u32::MAX,
                ..CPU_LOAD_DEFAULTS
            },
            CpuLoadSettings { batch_iterations: 0, ..CPU_LOAD_DEFAULTS },
            CpuLoadSettings {
                proportional_gain: f64::NAN,
                ..CPU_LOAD_DEFAULTS
            },
            CpuLoadSettings { integral_max: -1.0, ..CPU_LOAD_DEFAULTS },
        ] {
            assert!(settings.validate().is_err());
        }
    }

    #[test]
    fn out_of_bounds_probe_settings_are_refused() {
        for (field, maximum) in [
            ("system_interval_ms", 86_400_000),
            ("network_interval_ms", 86_400_000),
            ("wifi_link_interval_ms", 86_400_000),
            ("wifi_scan_interval_ms", 86_400_000),
            ("wifi_latency_interval_ms", 86_400_000),
            ("bluetooth_interval_ms", 86_400_000),
            ("minimum_scan_interval_s", 3_600_u64),
            ("system_probe_timeout_ms", 2_000),
            ("network_probe_timeout_ms", 2_000),
            ("cpu_calibration_ms", 1_000),
            ("wifi_link_probe_timeout_ms", 2_000),
            ("wifi_scan_probe_timeout_ms", 12_000),
            ("wifi_scan_window_ms", 6_000),
            ("wifi_latency_probe_timeout_ms", 4_000),
            ("wifi_echo_timeout_ms", 2_000),
            ("bluetooth_probe_timeout_ms", 6_000),
            ("bluetooth_deep_refresh_ms", 86_400_000),
            ("control_probe_timeout_ms", 30_000),
            ("radio_settle_timeout_ms", 15_000),
            ("minimum_rate_window_ms", 60_000),
            ("echo_payload_bytes", 65_000),
            ("calibration_batch_iterations", 65_536),
            ("period_ms", 60_000),
            ("sample_ms", 60_000),
            ("slice_ms", 1_000),
            ("settlement_ms", 60_000),
            ("batch_iterations", 65_536),
        ] {
            for value in [0, maximum + 1] {
                let mut configuration = dokime::plugin::Configuration::declared::<Settings>();
                configuration.settings.insert(field.into(), dokime::Value::from(value));
                assert!(configuration.settings::<Settings>().is_err(), "{field} accepted {value}");
            }
        }
        for value in [0.0, 100.01] {
            let mut configuration = dokime::plugin::Configuration::declared::<Settings>();
            configuration.settings.insert("congestion_network_weight".into(), dokime::Value::from(value));
            assert!(configuration.settings::<Settings>().is_err(), "congestion accepted {value}");
        }
    }
}
