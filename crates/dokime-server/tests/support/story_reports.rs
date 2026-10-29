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
use std::{collections::BTreeMap, path::Path};
// Third Party Imports
// Local Imports
use dokime::{
    client::{Client, TestExecutionHandle, contracts},
    domain::model,
};

/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
pub async fn finalize(client: &Client) {
    let root = tempfile::tempdir().unwrap();
    let mut request = contracts::execution::ListCycles {
        limit: 100,
        filter: Some(model::execution::wire::CycleFilter {
            archive: Some(model::execution::wire::ArchiveScope::All),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut saved = 0;
    loop {
        let page = client.execution_list_cycles(request.clone()).await.unwrap();
        saved += save_page(client, &page.cycles, root.path()).await;

        let Some(after) = page.next_after_cycle_id else {
            break;
        };
        request.after_cycle_id = after;
        request.cursor = page.cursor;
    }

    assert!(saved > 0, "report-producing system story has no executions; exempt no-report stories centrally");
    eprintln!("reopened {saved} sealed execution reports with their artifacts over native RPC");
}

/* === Private Functions === */
async fn save_page(client: &Client, cycles: &[model::execution::wire::CycleSummary], root: &Path) -> usize {
    let mut saved = 0;
    for cycle in cycles {
        saved += save_cycle(client, cycle, root).await;
    }
    saved
}

async fn save_cycle(client: &Client, cycle: &model::execution::wire::CycleSummary, root: &Path) -> usize {
    let cycle = client.cycle(dokime::TestCycleId::from_uuid(&cycle.cycle_id).unwrap());
    let executions = cycle.executions().await.unwrap();
    if executions.is_empty() {
        return 0;
    }
    cycle.seal().await.unwrap();

    let saved = executions.len();
    for execution in executions {
        save_execution(client, &execution, root).await;
    }
    saved
}

async fn save_execution(client: &Client, execution: &TestExecutionHandle, root: &Path) {
    let id = execution.id().to_string();
    assert!(execution.snapshot().await.unwrap().summary.unwrap().sealed);
    let checkpoint = execution.checkpoint().await.unwrap();
    let evidence = execution.evidence_at(&checkpoint).await.unwrap();
    let directory = root.join(&id);
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("report.html");

    let report = client
        .download_execution_report(contracts::reports::DownloadExecutionReport { execution_id: id.clone() }, path.clone())
        .await
        .unwrap();
    assert_eq!(report.metadata.origin, model::reports::wire::ReportOrigin::Sealed);
    super::system_reports::verify_file(&path, report.size_bytes, &report.sha256);

    let payload = super::system_reports::payload(&path);
    let mut artifacts = BTreeMap::new();
    for artifact in evidence.artifacts {
        let link = payload["snapshot"]["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|link| link["artifact_id"] == artifact.artifact_id)
            .expect("sealed artifact is in the report");
        let relative = link["path"].as_str().unwrap();
        dokime_runtime::validate_relative_output_path(relative).unwrap();
        let destination = directory.join("outputs").join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();

        let downloaded = client
            .download_artifact(
                contracts::content::DownloadArtifact {
                    artifact_id: artifact.artifact_id.clone(),
                },
                destination.clone(),
            )
            .await
            .unwrap();
        assert_eq!(downloaded.metadata, artifact);
        super::system_reports::verify_file(&destination, artifact.size_bytes, &artifact.sha256);
        assert!(
            artifacts
                .insert(
                    artifact.artifact_id,
                    super::system_reports::Artifact {
                        size_bytes: artifact.size_bytes,
                        sha256: artifact.sha256,
                    }
                )
                .is_none()
        );
    }
    super::system_reports::reopen(&path, &id, &artifacts);
    assert_eq!(execution.checkpoint().await.unwrap(), checkpoint);
    assert!(execution.snapshot().await.unwrap().summary.unwrap().sealed);
}

/* === Tests === */
