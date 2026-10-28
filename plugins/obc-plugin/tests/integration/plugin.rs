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

use obc_plugin::ObcPlugin;

#[tokio::test]
async fn resources_share_allocation() {
    use dokime::__harmos::Plugin;

    let mut plugin = <ObcPlugin as Plugin>::prepare(Default::default()).unwrap();
    let first = Plugin::resources(&plugin);
    let second = Plugin::resources(&plugin);
    assert!(std::sync::Arc::ptr_eq(&first, &second));

    let weak = std::sync::Arc::downgrade(&first);
    drop((first, second));
    plugin.cleanup().await.unwrap();
    drop(plugin);
    assert!(weak.upgrade().is_none());
}

#[test]
fn executable_declares_routes() {
    let declared =
        dokime::__harmos::sidecar::Executable::at(executable()).expect("the obc-plugin executable declares itself").declaration().clone();

    assert_eq!(declared.id, "obc");
    assert_eq!(declared.version.to_string(), "0.1.0");
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
                "obc.analysis.ripple",
                // Every template of every suite serves a run and a signal.
                "obc.charger.protection.overcurrent.run",
                "obc.charger.protection.overcurrent.signal",
                "obc.charger.recovery.power.run",
                "obc.charger.recovery.power.signal",
                "obc.charger.startup.sequence.run",
                "obc.charger.startup.sequence.signal",
                "obc.charger.steady.measurement.run",
                "obc.charger.steady.measurement.signal",
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
    assert_eq!("obc", declared.id);
}

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_obc-plugin")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_obc-plugin").map(str::to_owned))
        .expect("workspace executable")
}
