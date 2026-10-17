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
use obc_plugin::ObcPlugin;

/* === Constants === */
const EXECUTABLE: &str = env!("CARGO_BIN_EXE_obc-plugin");

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
    use dokime_runtime::__harmos::Plugin;

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
    let declared = dokime_runtime::__harmos::sidecar::Executable::at(EXECUTABLE)
        .expect("the obc-plugin executable declares itself")
        .declaration()
        .clone();

    assert_eq!(declared.id, "obc-plugin");
    assert_eq!(declared.version.to_string(), "0.1.0");
    let operations =
        declared.operations.iter().map(|operation| operation.id.as_str().to_owned()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        operations,
        std::collections::BTreeSet::from(
            [
                // The bench's own operations and its connection listener.
                "obc-plugin.check_readiness",
                "obc-plugin.connection",
                "obc-plugin.get_status",
                "obc-plugin.list_connections",
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
            "obc-plugin.system",
            "obc.charger.protection.samples",
            "obc.charger.recovery.samples",
            "obc.charger.startup.log",
            "obc.charger.startup.samples",
            "obc.charger.steady.events",
            "obc.charger.steady.samples",
            "obc.pulse",
        ]
    );
    assert_eq!("obc-plugin", declared.id);
}
