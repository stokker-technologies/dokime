/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Native Process Trace Export

/* === Modules === */

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

#[dokime_tracing::trace_test(logging)]
#[test]
fn payload_free_export() {
    use tracing_subscriber::Layer;
    let logging = tracing_subscriber::fmt::layer().with_writer(std::io::sink).with_filter(tracing_subscriber::filter::LevelFilter::WARN);
    let _logging = dokime_tracing::configure(vec![logging.boxed()]).unwrap();
    if std::env::var_os("DOKIME_TRACE_FIXTURE").is_some() {
        let parent = tracing::info_span!("parent", payload = "fixture-private-payload");
        let _parent = parent.enter();
        let child = tracing::info_span!("child", payload = "fixture-private-payload");
        let _child = child.enter();
        assert_eq!(tracing_error::SpanTrace::capture().status(), tracing_error::SpanTraceStatus::CAPTURED);
        tracing::info!(payload = "fixture-private-payload", "fixture-private-payload");
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let parent = tracing::Span::current();
                scope.spawn(move || {
                    let _parent = parent.enter();
                    for _ in 0..2048 {
                        let burst = tracing::debug_span!("burst", payload = "fixture-private-payload");
                        let _burst = burst.enter();
                    }
                });
            }
        });
        return;
    }
    let temporary = tempfile::tempdir().unwrap();
    let directory = std::env::var_os("DOKIME_TEST_TRACE_DIR").map(std::path::PathBuf::from).unwrap_or_else(|| temporary.path().to_owned());
    let existing = std::fs::read_dir(&directory).unwrap().map(|entry| entry.unwrap().path()).collect::<std::collections::BTreeSet<_>>();
    let executable = std::env::current_exe().unwrap();
    let child = dokime_domain::process::command(&executable)
        .args(["--exact", "process_export::payload_free_export", "--nocapture"])
        .env("DOKIME_TEST_TRACE_DIR", &directory)
        .env("DOKIME_TRACE_FIXTURE", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let process = executable.file_stem().unwrap().to_str().unwrap();
    let prefix = format!("{process}-{pid}-");
    let paths = std::fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            !existing.contains(path)
                && path.file_name().unwrap().to_string_lossy().starts_with(&prefix)
                && path.extension().is_some_and(|extension| extension == "pftrace")
        })
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 1);
    let text = std::fs::read_to_string(&paths[0]).unwrap();
    assert!(!text.contains("fixture-private-payload"));
    let trace: serde_json::Value = serde_json::from_str(&text).unwrap();
    let events = trace["traceEvents"].as_array().unwrap();
    let parent = events.iter().find(|event| event["name"] == "parent").unwrap();
    let child = events.iter().find(|event| event["name"] == "child").unwrap();
    assert_eq!(child["args"]["parent_id"], parent["args"]["span_id"]);
    assert!(child["dur"].as_f64().unwrap() <= parent["dur"].as_f64().unwrap());
    let burst = events.iter().filter(|event| event["name"] == "burst").collect::<Vec<_>>();
    assert_eq!(burst.len(), 4 * 2048, "flush retains every concurrently closed span");
    let ids = burst.iter().map(|event| event["args"]["span_id"].as_u64().unwrap()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), burst.len());
    assert!(burst.iter().all(|event| event["args"]["parent_id"] == child["args"]["span_id"]));
    assert!(events.iter().any(|event| event["name"] == "acceptance::process_export::payload_free_export"));
}
