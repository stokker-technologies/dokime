/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Mutating Radio, Adapter and Power Plan Controls, Gated on the Plugin's Durable Journal

/* === Modules === */

#[cfg(windows)]
mod platform {
    pub use crate::platform::windows::controls::{administrator, apply, bluetooth_adapter, connected_addresses, read_adapter, read_radio};
}

#[cfg(unix)]
mod platform {
    pub use crate::platform::unix::controls::{administrator, apply, bluetooth_adapter, connected_addresses, read_adapter, read_radio};
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;
use std::time::Duration;

// Third Party Imports
use prost_types::Timestamp;
use tokio::sync::Mutex;

// Local Imports
use crate::Host;
use crate::adapters::{Adapter, AdapterKind};
use crate::domain::controls::{AdapterState, ControlState, Radio, RadioPower};
use crate::settings::{self, CONTROL_PROBE_TIMEOUT, NETWORK_PROBE_TIMEOUT, RADIO_SETTLE_TIMEOUT};
use dokime::operations::{Context, Operation};
use dokime::plugin::Plugin;
use dokime::templates::Environment;
use dokime::{Error, Message, Result, Visibility};

/* === Constants === */

pub const RADIO_SET: &str = "host.network.radio.set";
pub const ADAPTER_SET: &str = "host.network.adapter.set";
pub const PLAN_SET: &str = "host.power.plan.set";
pub const RESTORE: &str = "host.controls.restore";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Controls State Retained Across Operation Calls: One Native Change at a Time.
#[derive(Default)]
pub struct State {
    gate: Mutex<()>,
}

/// Switch Every Radio of One Kind On or Off.
#[derive(Message)]
#[dokime(package = "host")]
pub struct RadioSet {
    pub radio: String,
    pub state: String,
    pub allow_disconnect: bool,
}

/// Enable or Disable One Network Adapter by Its Inventory Identity.
#[derive(Message)]
#[dokime(package = "host")]
pub struct AdapterSet {
    pub adapter_id: String,
    pub enabled: bool,
    pub allow_disconnect: bool,
}

/// Activate One Power Plan by Its Identity.
#[derive(Message)]
#[dokime(package = "host")]
pub struct PlanSet {
    pub plan_id: String,
}

/// Restore Every Outstanding Journal Inverse, Latest First.
#[derive(Message)]
#[dokime(package = "host")]
pub struct ControlsRestore {}

/// When the Control Settled; a Change That Was Already in Place Settles at Once.
#[derive(Message)]
#[dokime(package = "host")]
pub struct Applied {
    pub at: String,
}

/// The Native Waits Each Control Honors, Read From the Plugin Settings.
#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub control_probe: Duration,
    pub radio_settle: Duration,
    pub network_probe: Duration,
}

/// The Machine's Native Controls.
#[derive(Clone, Copy)]
pub struct Machine {
    pub timing: Timing,
}

/* === Enums === */

/// One Requested Control Change, Parsed From Its Operation Request.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Radio { radio: Radio, power: RadioPower, allow_disconnect: bool },
    Adapter { adapter_id: String, enabled: bool, allow_disconnect: bool },
    Plan { plan_id: String },
}

/* === Traits === */

/// The Native Reads and Writes a Control Performs; the Machine Implements It, Stories Fake It.
pub trait Native {
    fn administrator(&self) -> impl Future<Output = Result<bool>>;
    fn read(&self, change: &Change) -> impl Future<Output = Result<ControlState>>;
    fn apply(&self, state: &ControlState, restoring: bool) -> impl Future<Output = Result<()>>;
    /// The Adapters Carrying an Established Non-Loopback Connection of the Process.
    fn connected(&self, pid: u32) -> impl Future<Output = Result<Vec<Adapter>>>;
    fn radio_affects(&self, radio: Radio, adapter: &Adapter) -> Result<bool>;
}

/// The Durable Plugin Journal a Mutating Control Records Its Target and Inverse In Before It Writes.
pub trait Journal {
    /// Record the Inverse Unless the Target Already Has One; True When This Record Is the Target's Earliest. An Error Means the
    /// Journal Is Unavailable.
    fn record(&self, target: &str, inverse: &ControlState) -> impl Future<Output = Result<bool>>;
    /// Forget the Target Once Its Inverse Is Restored.
    fn clear(&self, target: &str) -> impl Future<Output = Result<()>>;
    /// Every Outstanding Target and Inverse in Record Order.
    fn outstanding(&self) -> impl Future<Output = Result<Vec<(String, ControlState)>>>;
}

/// What a Control Learns From Its Caller: the Time and, for the Connection Guard, the Backend Environment.
pub trait Caller {
    fn now(&self) -> impl Future<Output = Result<Timestamp>>;
    fn environment(&self) -> impl Future<Output = Result<Environment>>;
}

/* === Implementations === */

impl Change {
    pub fn radio(request: RadioSet) -> Result<Self> {
        let radio = Radio::parse(&request.radio)?;
        let power = match request.state.as_str() {
            "on" => RadioPower::On,
            "off" => RadioPower::Off,
            state => return Err(Error::Invalid(format!("Set Radio: Choose on or off Instead of {state:?}"))),
        };
        Ok(Self::Radio {
            radio,
            power,
            allow_disconnect: request.allow_disconnect,
        })
    }

    pub fn adapter(request: AdapterSet) -> Self {
        Self::Adapter {
            adapter_id: request.adapter_id,
            enabled: request.enabled,
            allow_disconnect: request.allow_disconnect,
        }
    }

    pub fn plan(request: PlanSet) -> Self {
        Self::Plan { plan_id: request.plan_id }
    }

    /// The State the Change Leaves Behind, Refusing a Read That Does Not Describe Its Target.
    pub fn after(&self, before: &ControlState) -> Result<ControlState> {
        let mut after = before.clone();
        match self {
            Self::Radio { radio, power, .. } => {
                let mixed = before.radios.iter().any(|state| state.kind != radio.as_str());
                if before.radios.is_empty() || mixed {
                    return Err(Error::Invalid(format!("Set Radio: Connect a {radio} Radio; None Is Available")));
                }
                for state in &mut after.radios {
                    state.on = *power == RadioPower::On;
                }
            }
            Self::Adapter { adapter_id, enabled, .. } => {
                let resolved = matches!(before.adapters.as_slice(), [state] if state.id == *adapter_id);
                if !resolved {
                    return Err(Error::Invalid(format!(
                        "Set Adapter: Choose an Identity From host.network.adapters Instead of {adapter_id}"
                    )));
                }
                after.adapters[0].enabled = *enabled;
            }
            Self::Plan { plan_id } => {
                if plan_id.is_empty() || before.power_plan.is_none() {
                    return Err(Error::Invalid("Set Power Plan: Name a Plan From host.power.plan.get on a Host With Power Plans".into()));
                }
                after.power_plan = Some(plan_id.clone());
            }
        }
        Ok(after)
    }

    /// The Journal Target, So Later Changes of One Target Keep Its Earliest Inverse.
    pub fn target(&self) -> String {
        match self {
            Self::Radio { radio, .. } => format!("host.radio.{radio}"),
            Self::Adapter { adapter_id, .. } => format!("host.adapter.{adapter_id}"),
            Self::Plan { .. } => "host.power.plan".into(),
        }
    }

    fn disabling(&self) -> bool {
        match self {
            Self::Radio { power, .. } => *power == RadioPower::Off,
            Self::Adapter { enabled, .. } => !enabled,
            Self::Plan { .. } => false,
        }
    }

    fn allow_disconnect(&self) -> bool {
        match self {
            Self::Radio { allow_disconnect, .. } | Self::Adapter { allow_disconnect, .. } => *allow_disconnect,
            Self::Plan { .. } => false,
        }
    }
}

impl State {
    /// Apply One Change: Read, Guard, Record the Inverse, Write, and Restore at Once When the Write Fails.
    pub async fn execute(&self, change: Change, native: &impl Native, journal: &impl Journal, caller: &impl Caller) -> Result<Applied> {
        let _gate = self.gate.lock().await;

        // Refuse Without Privilege
        if matches!(change, Change::Adapter { .. }) && !native.administrator().await? {
            return Err(Error::Invalid(format!("Set Adapter: Run the Host Plugin as Administrator to Call {ADAPTER_SET}")));
        }

        // Read Before and Compute the Inverse
        let before = native.read(&change).await?;
        let after = change.after(&before)?;
        if before == after {
            let at = caller.now().await?;
            return Ok(Applied { at: at.to_string() });
        }

        // Guard Active Connections
        if change.disabling() {
            let environment = caller.environment().await?;
            let overridden = change.allow_disconnect() && environment.in_step;
            let connected = if overridden { Vec::new() } else { native.connected(environment.backend_pid).await? };
            if cuts_connection(&change, &connected, |radio, adapter| native.radio_affects(radio, adapter))? {
                return Err(Error::Invalid(
                    "Set Control: Keep the Active Remote Client or Backend Connection, or Allow Disconnect From a Step".into(),
                ));
            }
        }

        // Record the Inverse, Then Write
        let target = change.target();
        let earliest = journal.record(&target, &before).await?;
        if let Err(error) = native.apply(&after, false).await {
            native.apply(&before, true).await.map_err(|restore| {
                Error::Invalid(format!("{error}; Restore the Control by Hand: the Immediate Inverse Failed: {restore}"))
            })?;
            if earliest {
                journal.clear(&target).await?;
            }
            return Err(error);
        }
        let at = caller.now().await?;
        Ok(Applied { at: at.to_string() })
    }

    /// Write Every Outstanding Inverse, Latest First, Clearing Each Once Written; a Failed Inverse Stays Outstanding.
    pub async fn restore(&self, native: &impl Native, journal: &impl Journal) -> Result<()> {
        let _gate = self.gate.lock().await;
        let outstanding = journal.outstanding().await?;
        for (target, inverse) in outstanding.into_iter().rev() {
            native.apply(&inverse, true).await?;
            journal.clear(&target).await?;
        }
        Ok(())
    }
}

impl Timing {
    pub async fn read(ctx: &Context) -> Result<Self> {
        Ok(Self {
            control_probe: settings::duration_value(ctx, CONTROL_PROBE_TIMEOUT).await?,
            radio_settle: settings::duration_value(ctx, RADIO_SETTLE_TIMEOUT).await?,
            network_probe: settings::duration_value(ctx, NETWORK_PROBE_TIMEOUT).await?,
        })
    }
}

impl Native for Machine {
    async fn administrator(&self) -> Result<bool> {
        blocking(platform::administrator).await
    }

    async fn read(&self, change: &Change) -> Result<ControlState> {
        let change = change.clone();
        let timing = self.timing;
        blocking(move || match change {
            Change::Radio { radio, .. } => platform::read_radio(radio, &timing),
            Change::Adapter { adapter_id, .. } => platform::read_adapter(&adapter_id).map(|enabled| ControlState {
                adapters: vec![AdapterState { id: adapter_id, enabled }],
                ..Default::default()
            }),
            Change::Plan { .. } => crate::power::active_plan().map(|plan| ControlState {
                power_plan: Some(plan.id),
                ..Default::default()
            }),
        })
        .await
    }

    async fn apply(&self, state: &ControlState, restoring: bool) -> Result<()> {
        let state = state.clone();
        let timing = self.timing;
        blocking(move || platform::apply(&state, restoring, &timing)).await
    }

    async fn connected(&self, pid: u32) -> Result<Vec<Adapter>> {
        let timing = self.timing;
        let locals = blocking(move || platform::connected_addresses(pid, &timing)).await?;
        let inventory = crate::adapters::snapshot(timing.network_probe).await?;
        let connected = inventory
            .adapters
            .into_iter()
            .filter(|adapter| adapter.ipv4.iter().chain(&adapter.ipv6).any(|address| locals.contains(&address.address)))
            .collect();
        Ok(connected)
    }

    fn radio_affects(&self, radio: Radio, adapter: &Adapter) -> Result<bool> {
        if radio == Radio::Bluetooth {
            return platform::bluetooth_adapter(adapter);
        }
        let kind = if radio == Radio::Cellular { AdapterKind::Cellular } else { AdapterKind::Wifi };
        Ok(adapter.is(kind))
    }
}

/// The Operation Context as the Caller.
impl Caller for Context {
    async fn now(&self) -> Result<Timestamp> {
        Context::now(self).await
    }

    async fn environment(&self) -> Result<Environment> {
        Context::environment(self).await
    }
}

/* === Public Functions === */

/// Declare the Controls Operations.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let radio = Operation::new(RADIO_SET, Visibility::Public, async |ctx: &Context, request: RadioSet| {
        let change = Change::radio(request)?;
        let machine = Machine { timing: Timing::read(ctx).await? };
        ctx.plugin::<Host>()?.controls.execute(change, &machine, &crate::journal::Journal::new(ctx), ctx).await
    })
    .policy(settings::policy(settings::CONTROL_BUDGET, Duration::ZERO))
    .description("Switch Every Radio of One Kind On or Off, Recording the Prior State for Restoration.");
    let adapter = Operation::new(ADAPTER_SET, Visibility::Public, async |ctx: &Context, request: AdapterSet| {
        let change = Change::adapter(request);
        let machine = Machine { timing: Timing::read(ctx).await? };
        ctx.plugin::<Host>()?.controls.execute(change, &machine, &crate::journal::Journal::new(ctx), ctx).await
    })
    .policy(settings::policy(settings::CONTROL_BUDGET, Duration::ZERO))
    .description("Enable or Disable One Network Adapter, Recording the Prior State for Restoration.");
    let plan = Operation::new(PLAN_SET, Visibility::Public, async |ctx: &Context, request: PlanSet| {
        let change = Change::plan(request);
        let machine = Machine { timing: Timing::read(ctx).await? };
        ctx.plugin::<Host>()?.controls.execute(change, &machine, &crate::journal::Journal::new(ctx), ctx).await
    })
    .policy(settings::policy(settings::CONTROL_BUDGET, Duration::ZERO))
    .description("Activate One Power Plan, Recording the Prior Plan for Restoration.");
    let restore = Operation::new(RESTORE, Visibility::Protected, async |ctx: &Context, _: ControlsRestore| {
        restore(ctx).await?;
        let at = ctx.now().await?;
        Ok(Applied { at: at.to_string() })
    })
    .policy(settings::policy(settings::RESTORE_BUDGET, Duration::ZERO))
    .description("Restore Every Control State a Mutating Control Recorded Before Its Write, Latest First.");
    Ok(plugin.operation(radio).operation(adapter).operation(plan).operation(restore))
}

/// Restore Every Outstanding Control State; the Restore Operation and the Plugin Start Both Run It.
pub async fn restore(ctx: &Context) -> Result<()> {
    let machine = Machine { timing: Timing::read(ctx).await? };
    ctx.plugin::<Host>()?.controls.restore(&machine, &crate::journal::Journal::new(ctx)).await
}

/// Whether Disabling the Change Cuts One of the Connected Adapters.
pub fn cuts_connection(change: &Change, connected: &[Adapter], radio_affects: impl Fn(Radio, &Adapter) -> Result<bool>) -> Result<bool> {
    for adapter in connected {
        let cut = match change {
            Change::Radio { radio, .. } => radio_affects(*radio, adapter)?,
            Change::Adapter { adapter_id, .. } => *adapter_id == adapter.id,
            Change::Plan { .. } => false,
        };
        if cut {
            return Ok(true);
        }
    }
    Ok(false)
}

/* === Private Functions === */

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    let task = tokio::task::spawn_blocking(work);
    task.await.map_err(|error| Error::Invalid(format!("Set Control: Retry After the Control Worker Stopped: {error}")))?
}

/* === Tests === */
