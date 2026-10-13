/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "../../tests/support/shutdown.rs"]
mod shutdown;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

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

fn benchmark_shutdown(c: &mut criterion::Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let plugin = std::env::var_os("DOKIME_BENCH_PLUGIN").map(std::path::PathBuf::from);
    let mut scenarios = vec![
        ("empty", false, false, None),
        ("idle-client", true, false, None),
        ("idle-watch", true, true, None),
    ];
    if let Some(plugin) = &plugin {
        scenarios.push(("idle-plugin", true, true, Some(plugin.as_path())));
    }
    let mut group = c.benchmark_group("shutdown");
    group.sample_size(10);
    for (name, connected, watching, plugin) in scenarios {
        group.bench_function(name, |b| {
            b.iter_custom(|iterations| {
                let mut total = std::time::Duration::ZERO;
                for _ in 0..iterations {
                    let elapsed = runtime.block_on(shutdown::measure_shutdown(
                        false,
                        connected,
                        watching,
                        plugin,
                        std::time::Duration::from_secs(2),
                    ));
                    assert!(
                        elapsed < std::time::Duration::from_secs(10),
                        "shutdown reached the slow-drain warning: {name}"
                    );
                    total += elapsed;
                }
                total
            })
        });
    }
    group.finish();
}
criterion::criterion_group!(benches, benchmark_shutdown);
criterion::criterion_main!(benches);
