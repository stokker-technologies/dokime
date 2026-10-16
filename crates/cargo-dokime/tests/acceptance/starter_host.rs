/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
use super::support;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};
// Third Party Imports
use serde_json::{Value, json};
// Local Imports
use dokime_client::{Client, Target, contracts};

/* === Constants === */
const READY_TIMEOUT: Duration = Duration::from_secs(60);
const EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Host {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
    _profile: Option<tempfile::TempDir>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Host {
    fn spawn(executable: &Path, bundle: &Path) -> Self {
        let profile = tempfile::tempdir().unwrap();
        let mut host = Self::start(executable, profile.path(), Some(bundle), "host");
        host._profile = Some(profile);
        host
    }

    fn start(executable: &Path, profile: &Path, bundle: Option<&Path>, run: &str) -> Self {
        let stdout = profile.join(format!("{run}.stdout.log"));
        let stderr = profile.join(format!("{run}.stderr.log"));
        let mut command = Command::new(executable);
        for (key, directory) in [
            ("LOCALAPPDATA", "profile/local"),
            ("XDG_CONFIG_HOME", "profile/config"),
            ("XDG_STATE_HOME", "profile/state"),
            ("XDG_DATA_HOME", "profile/data"),
        ] {
            command.env(key, profile.join(directory));
        }
        command
            .current_dir(profile)
            .env_remove("DOKIME_BRANDING_DIR")
            .args([
                "--dev-control",
                "--shutdown-stdin",
                "--no-seed-validation-artifact",
                "--port",
                "0",
            ]);
        if let Some(bundle) = bundle {
            command.arg("--plugin").arg(bundle);
        }
        let child = command
            .arg("--application-root")
            .arg(profile.join("application"))
            .arg("--output-root")
            .arg(profile.join("output"))
            .stdin(Stdio::piped())
            .stdout(fs::File::create(&stdout).unwrap())
            .stderr(fs::File::create(&stderr).unwrap())
            .spawn()
            .unwrap();
        Self {
            child,
            stdout,
            stderr,
            _profile: None,
        }
    }

    fn diagnostics(&self) -> String {
        format!(
            "{}\n{}",
            fs::read_to_string(&self.stdout).unwrap_or_default(),
            fs::read_to_string(&self.stderr).unwrap_or_default()
        )
    }

    async fn ready(&mut self) -> String {
        let deadline = Instant::now() + READY_TIMEOUT;
        loop {
            for line in fs::read_to_string(&self.stdout).unwrap_or_default().lines() {
                if let Ok(value) = serde_json::from_str::<Value>(line)
                    && value["event"] == "ready"
                {
                    assert_eq!(value["protocol"], dokime_contract::READINESS_PROTOCOL);
                    assert_eq!(value["transport"], "grpc");
                    return value["endpoint"].as_str().unwrap().to_owned();
                }
            }
            assert!(
                self.child.try_wait().unwrap().is_none(),
                "host exited before readiness: {}",
                self.diagnostics()
            );
            assert!(
                Instant::now() < deadline,
                "host readiness timed out: {}",
                self.diagnostics()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn reload(&mut self, bundle: &Path, rerun: bool) {
        use std::io::Write;
        let previous = fs::read_to_string(&self.stdout)
            .unwrap_or_default()
            .lines()
            .count();
        writeln!(
            self.child.stdin.as_mut().unwrap(),
            "{}",
            json!({
                "command": "reload", "path": bundle, "rerun": rerun,
            })
        )
        .unwrap();
        let deadline = Instant::now() + READY_TIMEOUT;
        loop {
            for line in fs::read_to_string(&self.stdout)
                .unwrap_or_default()
                .lines()
                .skip(previous)
            {
                if let Ok(reply) = serde_json::from_str::<Value>(line) {
                    assert_eq!(reply["event"], "reloaded", "{reply}");
                    if rerun {
                        assert_eq!(
                            reply["rerun_message"],
                            "Re-running the latest case with its retained inputs."
                        );
                    }
                    return;
                }
            }
            assert!(
                self.child.try_wait().unwrap().is_none(),
                "host exited: {}",
                self.diagnostics()
            );
            assert!(
                Instant::now() < deadline,
                "reload timed out: {}",
                self.diagnostics()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn shutdown(&mut self) {
        drop(self.child.stdin.take());
        let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(
                    status.success(),
                    "host shutdown failed: {}",
                    self.diagnostics()
                );
                return;
            }
            assert!(
                Instant::now() < deadline,
                "host ignored stdin EOF: {}",
                self.diagnostics()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        // Assertions must not leak a host or plugin. EOF remains the normal
        // shutdown path even during unwinding; kill and reap are a last resort.
        drop(self.child.stdin.take());
        let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
        while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

/* === Public Functions === */

/* === Private Functions === */
#[tokio::test]
async fn generated_plugin_lifecycle() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(|path| repository.join(path))
        .unwrap_or_else(|| repository.join("target"));
    let host = PathBuf::from(
        std::env::var_os("DOKIME_TEST_HOST").expect("mise selects the exact release host"),
    );
    assert!(
        host.is_file(),
        "run mise run build:release first: {}",
        host.display()
    );
    let target = target_root.join("starter-host");
    let outside = support::standalone_directory();
    let standalone = generate(outside.path(), "starter-soak", &repository);
    let bundle = build(&standalone, None, &target);
    inspect(&standalone, &bundle, "starter-soak");
    assert_installed(&host, &bundle, "starter-soak").await;
    assert_soak(&host, &bundle, "starter-soak", "passed").await;

    // A deliberately different default member makes package selection essential.
    // The helper remains local even in release mode; only Dokime must come from
    // the stamped release revision.
    let workspace = outside.path().join("workspace");
    fs::create_dir_all(workspace.join("helper/src")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"helper\"]\ndefault-members = [\"helper\"]\n",
    )
    .unwrap();
    fs::write(
        workspace.join("helper/Cargo.toml"),
        "[package]\nname = \"starter-helper\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    let helper = workspace.join("helper/src/lib.rs");
    fs::write(&helper, "pub fn band() -> f64 { 1.5 }\n").unwrap();
    let selected = generate(&workspace, "starter-selected", &repository);
    let manifest = selected.join("Cargo.toml");
    let mut document = fs::read_to_string(&manifest)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    let mut dependency = toml_edit::InlineTable::new();
    dependency.insert("path", "../helper".into());
    document["dependencies"]["starter-helper"] = toml_edit::value(dependency);
    fs::write(manifest, document.to_string()).unwrap();
    let hold = selected.join("src/suites/soak/hold.rs");
    let source = fs::read_to_string(&hold).unwrap();
    let authored = "if (row.celsius - target_c).abs() > band_c {";
    assert!(source.contains(authored));
    fs::write(
        hold,
        source.replace(
            authored,
            "if (row.celsius - target_c).abs() > starter_helper::band() {",
        ),
    )
    .unwrap();
    let bundle = build(&workspace, Some("starter-selected"), &target);
    inspect(&workspace, &bundle, "starter-selected");
    assert_soak(&host, &bundle, "starter-selected", "passed").await;

    // Rebuilding after an authored dependency edit must change executed behavior.
    // A band nothing can be inside puts every measured row out of it.
    fs::write(&helper, "pub fn band() -> f64 { -1.0 }\n").unwrap();
    let bundle = build(&workspace, Some("starter-selected"), &target);
    assert_soak(&host, &bundle, "starter-selected", "failed").await;
}

fn cli(root: &Path) -> Command {
    let mut command = Command::new(super::support::tool());
    command.current_dir(root).arg("dokime");
    command
}

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "command failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn generate(root: &Path, name: &str, repository: &Path) -> PathBuf {
    success(
        cli(root)
            .args(["new", name])
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .unwrap(),
    );
    let project = root.join(name);
    let manifest = project.join("Cargo.toml");
    let mut document = fs::read_to_string(&manifest)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    let sdk = &document["dependencies"]["dokime"];
    assert_eq!(
        sdk["git"].as_str(),
        Some("https://github.com/stokker-technologies/dokime")
    );
    assert_eq!(sdk["branch"].as_str(), Some("dev"));
    assert!(sdk.get("rev").is_none());
    let mut dependency = toml_edit::InlineTable::new();
    dependency.insert(
        "path",
        repository.join("crates/dokime").to_str().unwrap().into(),
    );
    document["dependencies"]["dokime"] = toml_edit::value(dependency);
    fs::write(manifest, document.to_string()).unwrap();
    project
}

fn build(root: &Path, package: Option<&str>, target: &Path) -> PathBuf {
    let bundle = root.join("fixture.dokime");
    let mut command = cli(root);
    command
        .arg("build")
        .arg("--target-dir")
        .arg(target)
        .arg("-o")
        .arg(&bundle);
    if let Some(package) = package {
        command.args(["-p", package]);
    }
    // Cargo gets no test deadline: only readiness after compilation is bounded.
    success(command.output().unwrap());
    bundle
}

fn inspect(root: &Path, bundle: &Path, package: &str) {
    let output = success(cli(root).arg("inspect").arg(bundle).output().unwrap());
    let inspection: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(inspection["manifest"]["identity"]["name"], package);
    let declarations = inspection["declarations"].as_object().unwrap();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations.values().next().unwrap()["id"], package);
}

async fn assert_soak(executable: &Path, bundle: &Path, package: &str, expected: &str) {
    let mut host = Host::spawn(executable, bundle);
    let endpoint = host.ready().await;
    host.reload(bundle, false).await;
    let catalog = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
        .await
        .unwrap();
    let planning = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
        .await
        .unwrap();
    let execution = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
        .await
        .unwrap();
    let suite_id = format!("{package}.soak");
    let suite = catalog
        .catalog_get_suite(contracts::catalog::GetSuiteRequest {
            suite_id: suite_id.clone(),
        })
        .await
        .unwrap();
    let template = suite
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == format!("{package}.soak.hold"))
        })
        .unwrap();
    let plan = contracts::planning::SelectionPlan {
        selections: vec![contracts::planning::RegisteredSelection {
            suite_id,
            template_id: template.template_id.clone(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let preview = planning
        .planning_preview_selection(contracts::planning::PreviewSelectionRequest {
            plan: Some(plan.clone()),
        })
        .await
        .unwrap();
    assert_eq!(preview.cases.len(), 1);
    let cycle_id = planned_cycle(&execution, "starter.host.soak", "Starter soak", plan).await;
    let before_cases = execution
        .execution_list_cases(contracts::execution::ListCasesRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(before_cases.next_after_case_id.is_none());
    let run = execution
        .execution_run_cycle(contracts::execution::RunCycleRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(matches!(
        run.outcome,
        Some(contracts::execution::RunReplyOutcome::Accepted(_))
    ));
    wait_terminal(&execution, &cycle_id).await;
    let attempts = execution
        .execution_list_executions(contracts::execution::ListExecutionsRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(attempts.next_after_execution_id.is_none());
    assert_eq!(attempts.executions.len(), 1);
    let first_id = attempts.executions[0].execution_id.clone();
    let mut first = execution
        .execution_get_execution(contracts::execution::GetExecutionRequest {
            execution_id: first_id.clone(),
        })
        .await
        .unwrap();
    first.observed_cursor = None;
    let expected_status = if expected == "passed" {
        contracts::execution::ExecutionStatus::Passed
    } else {
        contracts::execution::ExecutionStatus::Failed
    };
    if first.summary.as_ref().unwrap().status != expected_status {
        let health = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
            .await
            .unwrap()
            .health_list_health(contracts::notes::ListAnnotationsRequest {
                limit: 100,
                ..Default::default()
            })
            .await;
        panic!(
            "expected {expected_status:?}, execution: {first:#?}; health: {health:#?}; host: {}",
            host.diagnostics()
        );
    }
    assert_evidence(&endpoint, &first_id, package).await;
    let input_request = contracts::execution::ListExecutionParametersRequest {
        execution_id: first_id.clone(),
        set: contracts::execution::ExecutionParameterSet::Inputs,
        limit: 0,
        after_name: String::new(),
        cursor: None,
    };
    let inputs = execution
        .execution_list_execution_parameters(input_request.clone())
        .await
        .unwrap();
    assert!(inputs.next_after_name.is_none());
    let steps = execution
        .execution_list_execution_steps(contracts::execution::ListExecutionStepsRequest {
            execution_id: first_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(steps.next_cursor.is_none());
    let in_band = format!("{package}.soak.in_band");
    let mut decided = Vec::new();
    for step in steps.steps {
        let criteria = execution
            .execution_list_step_criteria(contracts::execution::ListStepEvidenceRequest {
                execution_id: first_id.clone(),
                step_id: step.step_id,
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(criteria.next_cursor.is_none());
        decided.extend(criteria.criteria.into_iter().filter(|criterion| {
            criterion
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == in_band)
        }));
    }
    assert_eq!(decided.len(), 1);
    assert_eq!(
        decided[0].status,
        if expected == "passed" {
            contracts::execution::CriterionStatus::Passed
        } else {
            contracts::execution::CriterionStatus::Failed
        }
    );
    host.reload(bundle, true).await;
    let deadline = Instant::now() + EXECUTION_TIMEOUT;
    loop {
        let repeated = execution
            .execution_list_executions(contracts::execution::ListExecutionsRequest {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(repeated.next_after_execution_id.is_none());
        if repeated.executions.len() == 2 && repeated.executions[1].status == expected_status {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "automatic re-run did not finish: {repeated:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    wait_terminal(&execution, &cycle_id).await;
    let mut retained = execution
        .execution_get_execution(contracts::execution::GetExecutionRequest {
            execution_id: first_id,
        })
        .await
        .unwrap();
    retained.observed_cursor = None;
    assert_eq!(
        retained, first,
        "re-run changed historical execution metadata"
    );
    let retained_inputs = execution
        .execution_list_execution_parameters(input_request)
        .await
        .unwrap();
    assert!(retained_inputs.next_after_name.is_none());
    assert_eq!(
        retained_inputs.parameters, inputs.parameters,
        "re-run changed frozen inputs"
    );
    let after_cases = execution
        .execution_list_cases(contracts::execution::ListCasesRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(after_cases.next_after_case_id.is_none());
    let selected_cases = |cases: &[contracts::execution::CaseSummary]| {
        cases
            .iter()
            .map(|case| {
                (
                    case.case_id.clone(),
                    case.template_id.clone(),
                    case.metadata.clone(),
                    case.ordinal,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        selected_cases(&after_cases.cases),
        selected_cases(&before_cases.cases),
        "re-run changed selected cases"
    );
    for case in &after_cases.cases {
        assert_eq!(case.attempt_count, 2);
        assert_eq!(case.latest_execution_status, Some(expected_status));
        assert!(!case.rerun_scheduled);
    }
    host.shutdown().await;
    assert!(
        catalog
            .catalog_get_suite(contracts::catalog::GetSuiteRequest {
                suite_id: format!("{package}.soak")
            })
            .await
            .is_err(),
        "host still accepts gRPC after shutdown"
    );
}

async fn assert_installed(executable: &Path, bundle: &Path, package: &str) {
    let profile = tempfile::tempdir().unwrap();
    let mut host = Host::start(executable, profile.path(), None, "install");
    let endpoint = host.ready().await;
    success(
        Command::new(super::support::tool())
            .args(["dokime", "--color", "never", "install"])
            .arg(bundle)
            .args(["--endpoint", &endpoint, "--yes"])
            .stdin(Stdio::null())
            .output()
            .unwrap(),
    );
    host.shutdown().await;

    let mut host = Host::start(executable, profile.path(), None, "installed");
    let endpoint = host.ready().await;
    let catalog = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
        .await
        .unwrap();
    let execution = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
        .await
        .unwrap();
    let suite_id = format!("{package}.soak");
    let suite = catalog
        .catalog_get_suite(contracts::catalog::GetSuiteRequest {
            suite_id: suite_id.clone(),
        })
        .await
        .unwrap_or_else(|status| panic!("{status}: {}", host.diagnostics()));
    let template = suite
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == format!("{package}.soak.hold"))
        })
        .unwrap();
    let plan = contracts::planning::SelectionPlan {
        selections: vec![contracts::planning::RegisteredSelection {
            suite_id,
            template_id: template.template_id.clone(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let cycle_id =
        planned_cycle(&execution, "starter.installed.soak", "Installed soak", plan).await;
    execution
        .execution_run_cycle(contracts::execution::RunCycleRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
    wait_terminal(&execution, &cycle_id).await;
    let attempts = execution
        .execution_list_executions(contracts::execution::ListExecutionsRequest {
            cycle_id,
            ..Default::default()
        })
        .await
        .unwrap()
        .executions;
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        attempts[0].status,
        contracts::execution::ExecutionStatus::Passed,
        "{attempts:#?}: {}",
        host.diagnostics()
    );
    assert_evidence(&endpoint, &attempts[0].execution_id, package).await;

    // The sensor port is discovered rather than saved: the host asks the
    // plugin's discovery operation once the plugin is ready and keeps the
    // answer beside the saved values.
    let settings = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
        .await
        .unwrap();
    let deadline = Instant::now() + EXECUTION_TIMEOUT;
    loop {
        let card = settings
            .settings_get_plugin_settings(contracts::settings::GetPluginSettingsRequest {
                plugin_id: package.to_owned(),
            })
            .await
            .unwrap();
        if card.discovered.as_ref().is_some_and(|discovered| {
            matches!(discovered.to_core().unwrap(), harmos::Value::Object(fields)
                if fields.keys().any(|id| id.ends_with("sensor_port")))
        }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "no discovered sensor port: {card:?}: {}",
            host.diagnostics()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    host.shutdown().await;
}

async fn assert_evidence(endpoint: &str, execution_id: &str, package: &str) {
    let scope = contracts::common::EvidenceScope {
        execution_id: Some(execution_id.to_owned()),
        ..Default::default()
    };
    let records = Client::connect(Target::local(endpoint.to_owned()).unwrap(), None, None)
        .await
        .unwrap()
        .evidence_list_records(contracts::evidence::ListRecordsRequest {
            selection: Some(contracts::evidence::CapturedRecordSelection {
                scope: Some(scope.clone()),
                record_types: vec![format!("{package}.soak.held")],
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(records.records.len(), 1, "{records:#?}");
    let held = records.records[0].details.as_ref().unwrap();
    assert!(
        matches!(
            held.to_core().unwrap(),
            harmos::Value::Object(fields) if matches!(fields.get("celsius"), Some(harmos::Value::Number(_)))
        ),
        "{held:#?}"
    );

    let samples = Client::connect(Target::local(endpoint.to_owned()).unwrap(), None, None)
        .await
        .unwrap()
        .telemetry_list_samples(contracts::telemetry::ListTelemetrySamplesRequest {
            streams: vec![format!("{package}.soak.samples")],
            scope: Some(scope),
            limit: 1000,
            ..Default::default()
        })
        .await
        .unwrap()
        .samples;
    let holding = samples
        .iter()
        .filter(|sample| {
            sample
                .values
                .as_ref()
                .map(|values| values.to_core().unwrap())
                .is_some_and(|value| {
                    matches!(value, harmos::Value::Object(fields)
                    if matches!(fields.get("holding"), Some(harmos::Value::Bool(true))))
                })
        })
        .count();
    assert!(holding > 0, "no held rows were captured: {samples:#?}");

    let files = Client::connect(Target::local(endpoint.to_owned()).unwrap(), None, None)
        .await
        .unwrap()
        .content_list_execution_files(contracts::content::ListExecutionFilesRequest {
            execution_id: execution_id.to_owned(),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap()
        .files;
    for artifact in ["heater.log", "summary.json"] {
        assert!(
            files
                .iter()
                .any(|file| file.relative_path.ends_with(artifact)),
            "{artifact} was not captured: {files:#?}"
        );
    }
}

async fn planned_cycle(
    execution: &Client,
    identifier: &str,
    label: &str,
    plan: contracts::planning::SelectionPlan,
) -> String {
    let cycle_id = execution
        .execution_create_cycle(contracts::execution::CreateCycleRequest {
            metadata: Some(contracts::common::Metadata {
                identifier: identifier.into(),
                label: label.into(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .cycle_id;
    execution
        .execution_append_planned_cases(contracts::execution::AppendPlannedCasesRequest {
            cycle_id: cycle_id.clone(),
            plan: Some(plan),
            context: None,
        })
        .await
        .unwrap();
    cycle_id
}

async fn wait_terminal(client: &Client, cycle_id: &str) {
    let deadline = Instant::now() + EXECUTION_TIMEOUT;
    loop {
        let cycle = client
            .execution_get_cycle(contracts::execution::GetCycleRequest {
                cycle_id: cycle_id.into(),
            })
            .await
            .unwrap();
        if matches!(
            cycle.status,
            contracts::execution::CycleStatus::Completed
                | contracts::execution::CycleStatus::Failed
                | contracts::execution::CycleStatus::Stopped
        ) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the soak did not complete: {cycle:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/* === Tests === */
