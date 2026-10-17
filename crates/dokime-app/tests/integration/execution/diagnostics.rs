/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::support::prelude::*;
use dokime::diagnostics::Diagnostic;
use dokime::domains::ExecutionProviderTarget;
use dokime::transport::ProviderExecutionFailure;
use dokime_app::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct DiagnosticExecutor {
    diagnostic: Diagnostic,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionFixture for DiagnosticExecutor {
    fn run_cycle<'a>(
        &'a self,
        _context: ExecutionFixtureContext,
        _cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            Err(ProviderExecutionFailure::new(
                "sidecar_execution",
                "step `hold` failed in thermo.soak.hold",
            )
            .detailed(&self.diagnostic))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn stored_provider_paths() {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        });
    assert!(!home.is_empty());
    let diagnostic = Diagnostic {
        message: format!("opening {home}/bench/thermo.toml failed"),
        chain: vec![format!("{home}/bench is not readable")],
        location: format!("{home}/bench/src/hold.rs:71"),
        span_trace: vec![format!("step root={home}/bench")],
        backtrace: format!("   0: thermo_bench::hold\n     at {home}/bench/src/hold.rs:71"),
    };
    let root = tempdir().unwrap();
    let (mut owner, handle) =
        booted_with_fixture(root.path(), Arc::new(DiagnosticExecutor { diagnostic })).await;
    let suite = fixture_suite(&["failed"]);
    register_python_suite(&handle, suite.clone()).await;
    let cycle = start_cycle(&handle, suite, false).await;
    let mut records = handle.subscribe_records();
    handle
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    wait_for_cycle_lifecycle(&handle, &cycle, CycleStatus::Failed).await;
    wait_for_subscription_record(&mut records, "provider.execution.failed").await;
    let evidence = handle.evidence_snapshot().await.unwrap();
    let snapshot = handle.watch_state().latest();
    assert!(!snapshot.state().installed_plugin_errors.is_empty());
    assert!(
        evidence
            .records
            .iter()
            .any(|record| record.record_type.as_str() == "provider.execution.failed")
    );
    let stored = format!(
        "{}{}",
        serde_json::to_string(&evidence.records).unwrap(),
        serde_json::to_string(&snapshot.state().installed_plugin_errors).unwrap()
    );
    assert!(!stored.contains(&home), "{stored}");
    assert!(
        stored.contains("is not readable"),
        "redaction must retain the diagnostic cause"
    );
    owner.shutdown().await.unwrap();
}
