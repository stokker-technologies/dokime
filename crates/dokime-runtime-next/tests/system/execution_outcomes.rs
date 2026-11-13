/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Outcome Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_protocol::v1;
use dokime_runtime_next::{Host, PluginProcess};
use dokime_storage::OutcomeCounts;

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
async fn execution_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Materialize an Evaluated and a Criterion-Free Execution
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let executable = super::support::execution_fixture()?;
        let host = Host::open(root.path(), settings.clone())?;
        let mut plugin = PluginProcess::start(&executable, true, &settings.placement).await?;
        let entry = host.add_remote(plugin.endpoint()).await?;
        let stale = host.materialize(&entry, "execution.capture", super::support::parameters(8.0, 0, true)).await?;
        let unevaluated = host.materialize(&entry, "execution.capture", super::support::parameters(9.0, 0, false)).await?;
        let cycle = host.start_cycle(vec![stale.clone(), unevaluated]).await?;
        let cycle_identifier = cycle.identifier.clone();
        for execution in &cycle.executions {
            let prompt = host.next_prompt(execution).await?;
            super::support::answer(&host, execution, prompt)?;
        }
        let result = cycle.wait().await?;
        assert_eq!(
            result.counts,
            OutcomeCounts {
                evaluated: 1,
                not_evaluated: 1,
                declared: 0,
                verdict: Some(v1::ResultStatus::Passed)
            }
        );
        assert!(matches!(
            result.executions[1].execution.outcome.as_ref().unwrap().attribution,
            Some(v1::execution_outcome::Attribution::NotEvaluated(_))
        ));

        // Count Declared Outcomes Without Letting Them Set the Cycle Verdict
        let declared = v1::ExecutionOutcome {
            attribution: Some(v1::execution_outcome::Attribution::Declared(v1::DeclaredOutcome {
                status: v1::ResultStatus::Failed as i32,
                importer: "sample.importer".into(),
                ..Default::default()
            })),
        };
        let store = dokime_storage::Store::open(root.path())?;
        store.create(&dokime_storage::Execution {
            identifier: "declared".into(),
            cycle: cycle_identifier.clone(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import {
                    importer: "sample.importer".into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        })?;
        store.close("declared", v1::Termination::Completed, declared.clone())?;
        store.seal("declared")?;
        let mut outcomes = result.executions.iter().map(|snapshot| snapshot.execution.outcome.as_ref().unwrap()).collect::<Vec<_>>();
        outcomes.push(&declared);
        assert_eq!(
            OutcomeCounts::from_outcomes(outcomes),
            OutcomeCounts {
                evaluated: 1,
                not_evaluated: 1,
                declared: 1,
                verdict: Some(v1::ResultStatus::Passed)
            }
        );
        assert_eq!(
            host.cycle_counts(&cycle_identifier)?,
            OutcomeCounts {
                evaluated: 1,
                not_evaluated: 1,
                declared: 1,
                verdict: Some(v1::ResultStatus::Passed)
            }
        );
        drop(store);
        assert_eq!(OutcomeCounts::from_outcomes([&declared]).verdict, None);
        assert_eq!(OutcomeCounts::from_outcomes([result.executions[1].execution.outcome.as_ref().unwrap()]).verdict, None);
        assert_eq!(OutcomeCounts::from_outcomes([]).verdict, None);

        // Replace the Catalog and Rematerialize Instead of Reusing the Stale Case
        host.remove_plugin(&entry).await?;
        plugin.close().await?;
        let mut revised = PluginProcess::start_with_environment(
            &executable,
            true,
            &settings.placement,
            &[("DOKIME_EXECUTION_REVISION".into(), "2".into())],
        )
        .await?;
        let entry = host.add_remote(revised.endpoint()).await?;
        let cycle = host.start_cycle(vec![stale.clone()]).await?;
        let prompt = host.next_prompt(&cycle.executions[0]).await?;
        super::support::answer(&host, &cycle.executions[0], prompt)?;
        let result = cycle.wait().await?;
        let Some(v1::origin::Kind::PluginRun(current)) = result.executions[0].execution.origin.as_ref().unwrap().kind.as_ref() else {
            panic!("PluginRun")
        };
        let Some(v1::origin::Kind::PluginRun(previous)) = stale.kind.as_ref() else { panic!("PluginRun") };
        assert_ne!(current.digest, previous.digest);
        assert_eq!(current.digest, host.description(&entry)?.catalog.unwrap().digest);
        assert_eq!(current.test_case.as_ref().unwrap().steps.len(), 2);
        assert_eq!(previous.test_case.as_ref().unwrap().steps.len(), 3);
        assert!(!result.executions[0].evidence.iter().any(|evidence| evidence.step == "tail"));

        // Refuse a Failed Rematerialization Without Creating Execution Evidence
        let before = host.executions()?;
        let mut invalid = stale;
        let Some(v1::origin::Kind::PluginRun(run)) = invalid.kind.as_mut() else { unreachable!() };
        run.test_case.as_mut().unwrap().parameters = super::support::parameters(8.0, u32::MAX, true);
        assert!(host.start_cycle(vec![invalid]).await.is_err());
        assert_eq!(host.executions()?, before);
        host.shutdown().await?;
        drop(host);

        // Read Mixed Attributions and Counts Through the Client Services Once the Root Is Released
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut query = v1::query_client::QueryClient::connect(process.url.clone()).await?;
        let page = query
            .list_executions(v1::QueryListExecutionsRequest {
                selection: Some(v1::ExecutionSelection {
                    cycle_ids: vec![cycle_identifier.clone()],
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?
            .into_inner();
        let counts = page.outcome_counts.unwrap();
        assert_eq!((counts.evaluated_count, counts.not_evaluated_count, counts.declared_count), (1, 1, 1));
        assert_eq!(counts.result_status, Some(v1::ResultStatus::Passed as i32));
        assert!(page.rows.iter().any(|row| matches!(
            row.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()),
            Some(v1::execution_outcome::Attribution::Declared(_))
        )));
        let mut reader = v1::execution_client::ExecutionClient::connect(process.url.clone()).await?;
        for row in &page.rows {
            let detail =
                reader.get_execution(v1::ExecutionGetExecutionRequest { execution_id: row.execution_id.clone() }).await?.into_inner();
            assert_eq!(detail.cycle_counts.as_ref(), Some(&counts));
            assert_eq!(detail.summary.as_ref(), Some(row));
            assert!(detail.controls.as_ref().unwrap().can_inspect);
        }
        process.close().await?;
        revised.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
