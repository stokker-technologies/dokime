/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The plugin's resources and the declaration its built executable carries,
// inspected without starting it.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use plugin_obc::ObcPlugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn resources_share_allocation() {
    use dokime::harmos::Lifecycle;
    let mut setup = dokime::harmos::Setup::<ObcPlugin>::new(harmos_sdk::__private::WireTable::new("plugin_obc").unwrap());
    let mut plugin = <ObcPlugin as Lifecycle>::prepare(Default::default(), &mut setup).await.unwrap();
    let first = setup.settings().context().resource::<plugin_obc::Resources>().unwrap();
    let second = setup.settings().context().resource::<plugin_obc::Resources>().unwrap();
    assert!(std::sync::Arc::ptr_eq(&first, &second));

    let weak = std::sync::Arc::downgrade(&first);
    drop((first, second));
    plugin.cleanup().await.unwrap();
    drop(plugin);
    drop(setup);
    assert!(weak.upgrade().is_none());
}

#[test]
fn executable_declares_routes() {
    // Workspace Feature Unification May Enable Host Implementations, But Only The Host Binary Declares That Sidecar.
    let declared =
        dokime::__harmos::sidecar::Executable::at(executable()).expect("the plugin-obc executable declares itself").declaration().clone();

    assert_eq!(declared.id, "plugin_obc");
    assert_eq!(declared.version.to_string(), env!("CARGO_PKG_VERSION"));
    let operations =
        declared.operations.iter().map(|operation| operation.id.as_str().to_owned()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        operations,
        std::collections::BTreeSet::from(
            [
                // The bench's own operations and its connection listener.
                "obc.bench.check_readiness",
                "obc.bench.connection",
                "obc.bench.get_status",
                "obc.bench.list_connections",
                "obc.device.read",
                "obc.device.set_power",
                // The ripple analysis the plugin contributes is an operation too.
                "plugin_obc.analysis.ripple",
                // Every template of every suite serves a run and a signal.
                "obc.charger.protection.overcurrent.run",
                "obc.charger.protection.overcurrent.signal",
                "obc.charger.recovery.power.run",
                "obc.charger.recovery.power.signal",
                "obc.charger.startup.sequence.run",
                "obc.charger.startup.sequence.signal",
                "obc.charger.steady.measurement.run",
                "obc.charger.steady.measurement.signal",
                "obc.checks.cpu_load.run",
                "obc.checks.cpu_load.signal",
                "obc.checks.pulse.run",
                "obc.checks.pulse.signal",
            ]
            .map(str::to_owned)
        )
    );
    let streams = declared.streams.iter().map(|stream| stream.id.as_str().to_owned()).collect::<Vec<_>>();
    assert_eq!(
        streams,
        [
            "obc.charger.protection.samples",
            "obc.charger.recovery.samples",
            "obc.charger.startup.log",
            "obc.charger.startup.samples",
            "obc.charger.steady.events",
            "obc.charger.steady.samples",
            "obc.pulse",
            "obc.system",
        ]
    );
    assert_eq!("plugin_obc", declared.id);
}

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_plugin-obc")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_plugin-obc").map(str::to_owned))
        .expect("workspace executable")
}
