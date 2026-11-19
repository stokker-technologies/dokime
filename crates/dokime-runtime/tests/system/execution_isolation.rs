/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Concurrent Execution Isolation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;
use std::sync::{Arc, Mutex};

// Third Party Imports
use harmos_protocol::host::{OperationsGetRequest, OperationsListRequest, PluginState, plugin_placement};
use harmos_protocol::model::{
    Callers, Classification, Declarations, Failure, OperationContract, OperationPolicy, Position, Requirement, Section, StreamDeclaration,
    Visibility,
};
use harmos_protocol::plugin::{
    ChildRequest, InstanceGetRequest, InstanceGetResponse, InstanceInvokeRequest, InstanceInvokeResponse, InstanceSamplesRequest,
    InstanceSamplesResponse, InstanceStopRequest, InstanceStopResponse, child_response, instance_invoke_request, instance_invoke_response,
    instance_server::{Instance, InstanceServer},
    response,
};
use harmos_protocol::schema;
use harmos_runtime::operations::OperationFactory;
use prost::Message;
use prost_reflect::ReflectMessage;
use prost_types::{Duration, FileDescriptorSet, Timestamp};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::time;
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::transport::{Endpoint, Server};
use tonic::{Code, Request, Response, Status, Streaming};
use uuid::Uuid;

// Local Imports
use super::{client_runs_thermal_suite_over_grpc::HostProcess, support};
use dokime::client::{ExecutionMode, ExecutionOutcome, PluginPlacement, PluginRemote, PluginValue, ResultStatus};
use dokime::record_scope::Kind;
use dokime::{
    ArtifactScope, Client, EvidenceScope, FileReference, Import, Metadata, Origin, RecordScope, Step, Suite, Suites, Termination, TestCase,
    TestCaseTemplate, origin, value,
};
use dokime_operations::host;
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::evidence::{
    LogsAppendRequest, RecordsAppendRequest, RecordsListRequest, TraceabilityGetRequest, logs_client::LogsClient,
};
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_protocol::host::{EvidenceStatisticsRequest, EvidenceStatisticsResponse, SettingsGetRequest, execution_outcome};
use dokime_protocol::plugin::{StepsExecuteRequest, StepsExecuteResponse, TestCasesMaterializeRequest, TestCasesMaterializeResponse};
use dokime_runtime::{Host, services};
use dokime_storage::{Execution, Sample, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
struct ScopeProbe {
    description: InstanceGetResponse,
    store: Store,
    file: Arc<Mutex<Option<FileReference>>>,
    observations: Arc<Mutex<Vec<(String, bool, bool)>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Instance for ScopeProbe {
    async fn get(&self, _: Request<InstanceGetRequest>) -> Result<Response<InstanceGetResponse>, Status> {
        Ok(Response::new(self.description.clone()))
    }
    type InvokeStream = ReceiverStream<Result<InstanceInvokeResponse, Status>>;
    async fn invoke(&self, request: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        let mut input = request.into_inner();
        let Some(instance_invoke_request::Frame::Request(request)) = input.message().await?.and_then(|frame| frame.frame) else {
            return Err(Status::invalid_argument("Open Scope Probe: Supply an Invocation"));
        };
        let (send, receive) = mpsc::channel(1);
        let probe = self.clone();
        tokio::spawn(async move {
            let result = async {
                // Materialize a Complete Frozen Case Without Calling the Host
                if request.operation == "scope_fixture.materialize" {
                    let request = TestCasesMaterializeRequest::decode(request.payload.as_slice())?;
                    let reply = TestCasesMaterializeResponse {
                        test_case: Some(TestCase {
                            parameters: request.parameters,
                            steps: vec![Step {
                                metadata: Some(Metadata {
                                    identifier: "probe".into(),
                                    ..Default::default()
                                }),
                                operation: "scope_fixture.capture".into(),
                                repeatable: false,
                            }],
                            ..Default::default()
                        }),
                    };
                    return Ok(reply.encode_to_vec());
                }
                let step = StepsExecuteRequest::decode(request.payload.as_slice())?;
                assert_eq!(request.scope, step.execution);

                // Seed Exact Evidence for This Execution Before Testing Reads
                probe.store.open_stream(&step.execution, "probe.samples")?;
                probe.store.sample(
                    &step.execution,
                    Sample {
                        stream: "probe.samples".into(),
                        source_ns: 0,
                        sequence: 1,
                        row: Position { history: None, sequence: u64::MAX }.encode_to_vec(),
                        ..Default::default()
                    },
                )?;
                probe.store.close_streams(&step.execution)?;
                let file = probe.file.lock().unwrap().clone().ok_or("Publish Scope Probe File Before Execution")?;
                let mut cases = Vec::new();
                for selected in [None, Some(step.execution.clone()), Some("foreign-execution".into())] {
                    let foreign = selected.as_deref() == Some("foreign-execution");
                    let artifact = ArtifactsCreateRequest {
                        file: Some(file.clone()),
                        scope: selected.clone().map(|execution_id| ArtifactScope {
                            execution_id: Some(execution_id),
                            ..Default::default()
                        }),
                        filename: "probe.bin".into(),
                        content_type: "application/octet-stream".into(),
                        request_id: Uuid::now_v7().to_string(),
                        ..Default::default()
                    };
                    let scope = selected.clone().map(|execution_id| EvidenceScope {
                        execution_id: Some(execution_id),
                        ..Default::default()
                    });
                    let statistics = EvidenceStatisticsRequest {
                        scope: scope.clone(),
                        stream: "probe.samples".into(),
                        field: "sequence".into(),
                        ..Default::default()
                    };
                    let records = RecordsListRequest {
                        scope: selected.clone().map(|execution| RecordScope { kind: Some(Kind::Execution(execution)) }),
                        ..Default::default()
                    };
                    let traceability = TraceabilityGetRequest { scope, ..Default::default() };
                    let progress = ProgressGetRequest { name: selected, ..Default::default() };
                    cases.extend([
                        ("dokime.host.v1.content.Artifacts/Create", artifact.encode_to_vec(), foreign, false),
                        ("dokime.host.v1.Evidence/Statistics", statistics.encode_to_vec(), foreign, false),
                        ("dokime.host.v1.evidence.Records/List", records.encode_to_vec(), foreign, false),
                        ("dokime.host.v1.evidence.Traceability/Get", traceability.encode_to_vec(), foreign, false),
                        ("dokime.host.v1.executions.Progress/Get", progress.encode_to_vec(), foreign, false),
                    ]);
                }
                cases.extend([
                    ("dokime.host.v1.Settings/Get", SettingsGetRequest::default().encode_to_vec(), true, true),
                    ("harmos.host.v1.Operations/List", OperationsListRequest::default().encode_to_vec(), true, true),
                    (
                        "harmos.host.v1.Operations/Get",
                        OperationsGetRequest { id: "harmos.host.v1.Time/Get".into() }.encode_to_vec(),
                        true,
                        true,
                    ),
                ]);

                // Exercise Each Request Over the Real Plugin Child Carrier
                let types = probe.description.declarations.as_ref().unwrap().types.as_ref().unwrap();
                for (index, (operation, payload, refused, wrong_caller)) in cases.into_iter().enumerate() {
                    let (service, method) = operation.split_once('/').unwrap();
                    let descriptor = dokime_protocol::descriptor_pool()
                        .get_service_by_name(service)
                        .unwrap()
                        .methods()
                        .find(|candidate| candidate.name() == method)
                        .unwrap();
                    let fingerprint =
                        schema::fingerprint(operation, types, descriptor.input().full_name(), descriptor.output().full_name())?;
                    let call = index as u64 + 1;
                    let frame = InstanceInvokeResponse {
                        frame: Some(instance_invoke_response::Frame::Child(ChildRequest {
                            call,
                            operation: operation.into(),
                            payload,
                            fingerprint,
                            ..Default::default()
                        })),
                    };
                    send.send(Ok(frame)).await.map_err(|_| "Send Scope Probe: Carrier Closed")?;
                    let Some(instance_invoke_request::Frame::Child(reply)) = input.message().await?.and_then(|frame| frame.frame) else {
                        return Err("Read Scope Probe: Missing Child Reply".into());
                    };
                    assert_eq!(reply.call, call);
                    let rejected = matches!(reply.result, Some(child_response::Result::Failure(_)));
                    probe.observations.lock().unwrap().push((operation.into(), refused, rejected));
                    if operation == "dokime.host.v1.Evidence/Statistics" && !refused {
                        let Some(child_response::Result::Payload(payload)) = reply.result.as_ref() else {
                            return Err(format!("{operation}: Refused the Calling Execution: {:?}", reply.result).into());
                        };
                        let statistics = EvidenceStatisticsResponse::decode(payload.as_slice())?.statistics.unwrap();
                        for value in [statistics.min, statistics.max, statistics.mean, statistics.sum] {
                            match value.unwrap().kind.unwrap() {
                                value::Kind::UnsignedInteger(value) => assert_eq!(value, u64::MAX),
                                value::Kind::Exact(value) => {
                                    assert!(!value.negative);
                                    assert_eq!(value.numerator, u64::MAX.to_string());
                                    assert_eq!(value.denominator, "1");
                                }
                                value => return Err(format!("{operation}: Lost Integer Precision: {value:?}").into()),
                            }
                        }
                    }
                    if wrong_caller {
                        let Some(child_response::Result::Failure(failure)) = reply.result else {
                            return Err(format!("{operation}: Accepted a Plugin Caller").into());
                        };
                        assert_eq!(failure.classification, Classification::Refused as i32, "{operation}: {failure:?}");
                        assert_eq!(failure.code, "-32604", "{operation}: {failure:?}");
                        assert_eq!(failure.action, "Call", "{operation}: {failure:?}");
                    }
                }
                Ok::<Vec<u8>, Box<dyn Error + Send + Sync>>(StepsExecuteResponse {}.encode_to_vec())
            }
            .await;
            let result = match result {
                Ok(payload) => response::Result::Payload(payload),
                Err(error) => response::Result::Failure(Failure {
                    classification: Classification::Permanent as i32,
                    message: error.to_string(),
                    ..Default::default()
                }),
            };
            let frame = InstanceInvokeResponse {
                frame: Some(instance_invoke_response::Frame::Response(harmos_protocol::plugin::Response { result: Some(result) })),
            };
            let _ = send.send(Ok(frame)).await;
        });
        Ok(Response::new(ReceiverStream::new(receive)))
    }
    type SamplesStream = ReceiverStream<Result<InstanceSamplesResponse, Status>>;
    async fn samples(&self, request: Request<Streaming<InstanceSamplesRequest>>) -> Result<Response<Self::SamplesStream>, Status> {
        let mut input = request.into_inner();
        let (send, receive) = mpsc::channel(1);
        tokio::spawn(async move {
            // The Probe's Device Stream Publishes Nothing Itself; Steps Seed Its Evidence Directly.
            while let Ok(Some(_)) = input.message().await {}
            drop(send);
        });
        Ok(Response::new(ReceiverStream::new(receive)))
    }
    async fn stop(&self, _: Request<InstanceStopRequest>) -> Result<Response<InstanceStopResponse>, Status> {
        Ok(Response::new(InstanceStopResponse {}))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn encoded_admission_refuses_plugin_only_operations_for_clients() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Start the Runtime's Actual Client Admission Path
            let root = support::tempdir()?;
            let server = services::serve("127.0.0.1:0".parse()?, root.path(), support::settings(), support::service_settings()).await?;
            let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let types = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?;
            let mut checked = 0;

            // Call Every Plugin-Only Product Operation Using Its Declared Fingerprint
            for service in
                dokime_protocol::descriptor_pool().services().filter(|service| service.package_name().starts_with("dokime.host.v1"))
            {
                for method in service.methods() {
                    if schema::method_callers(&method)? != Callers::Plugins {
                        continue;
                    }
                    let operation = format!("{}/{}", service.full_name(), method.name());
                    let fingerprint = schema::fingerprint(&operation, &types, method.input().full_name(), method.output().full_name())?;
                    let status = client.invocations.create(&operation, fingerprint, Vec::new()).await.unwrap_err();
                    assert_eq!(status.code(), Code::PermissionDenied, "{operation}: {status}");
                    let failure = Failure::decode(status.details())?;
                    assert_eq!(failure.classification, Classification::Refused as i32, "{operation}: {failure:?}");
                    assert_eq!(failure.code, "-32604", "{operation}: {failure:?}");
                    assert_eq!(failure.action, "Call", "{operation}: {failure:?}");
                    checked += 1;
                }
            }
            assert!(checked > 0, "Exercise Client Admission: Register Plugin-Only Operations");
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn clients_cannot_call_plugin_operations_over_typed_or_encoded_grpc() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        time::timeout(support::SYSTEM_HANG_GUARD, async {
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), support::client_settings())?;
            let operation = "dokime.host.v1.evidence.Logs/Append";
            // Use the Local Contract Excluded from Client Discovery
            let factory = OperationFactory::protobuf::<host::evidence::logs::append::Operation>()?;
            let contract = factory.contract()?;
            let request = LogsAppendRequest::default();
            let typed = LogsClient::new(channel).append(request.clone()).await.unwrap_err();
            let encoded =
                client.invocations.create(operation, contract.fingerprint().0.to_vec(), request.encode_to_vec()).await.unwrap_err();
            for status in [typed, encoded] {
                assert_eq!(status.code(), Code::PermissionDenied, "{status}");
                let failure = Failure::decode(status.details())?;
                assert_eq!(failure.classification, Classification::Refused as i32);
                assert_eq!(failure.code, "-32604");
                assert_eq!(failure.action, "Call");
            }
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn harmonized_plugin_calls_retain_trusted_execution_scope() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Freeze the Plugin's Lifecycle Contracts and Product Contributions
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let store = Store::open(root.path())?;
            store.create(&Execution {
                identifier: "foreign-execution".into(),
                cycle: "foreign-cycle".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
                }),
                ..Default::default()
            })?;
            store.close_streams("foreign-execution")?;
            store.close("foreign-execution", Termination::Completed, ExecutionOutcome::default())?;
            store.seal("foreign-execution", Default::default())?;
            let types = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?;
            let mut operations = Vec::new();
            for (id, request, response) in [
                (
                    "scope_fixture.materialize",
                    TestCasesMaterializeRequest::default().descriptor(),
                    TestCasesMaterializeResponse::default().descriptor(),
                ),
                ("scope_fixture.capture", StepsExecuteRequest::default().descriptor(), StepsExecuteResponse::default().descriptor()),
            ] {
                operations.push(OperationContract {
                    id: id.into(),
                    request: request.full_name().into(),
                    response: response.full_name().into(),
                    fingerprint: schema::fingerprint(id, &types, request.full_name(), response.full_name())?,
                    visibility: Visibility::Protected as i32,
                    policy: Some(OperationPolicy {
                        timeout: Some(Duration::try_from(support::OPERATION_TIMEOUT)?),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            }
            let suites = Suites {
                suites: vec![Suite {
                    metadata: Some(Metadata {
                        identifier: "scope_fixture".into(),
                        ..Default::default()
                    }),
                    templates: vec![TestCaseTemplate {
                        metadata: Some(Metadata {
                            identifier: "scope_fixture.capture".into(),
                            ..Default::default()
                        }),
                        materialize: "scope_fixture.materialize".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            // Read Required Contracts from the Active Host Registration
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let mut host_contracts = Vec::new();
            let required_operations = [
                "dokime.host.v1.content.Artifacts/Create",
                "dokime.host.v1.Evidence/Statistics",
                "dokime.host.v1.evidence.Records/List",
                "dokime.host.v1.evidence.Traceability/Get",
                "dokime.host.v1.executions.Progress/Get",
                "dokime.host.v1.Settings/Get",
            ];
            for id in required_operations.into_iter().chain(["harmos.host.v1.Operations/List", "harmos.host.v1.Operations/Get"]) {
                let contract = client.operations.get(id).await?.contract.ok_or("Discover Required Operation: Missing Contract")?;
                host_contracts.push(contract);
            }
            let requires: Vec<Requirement> = host_contracts
                .iter()
                .filter(|contract| required_operations.contains(&contract.id.as_str()))
                .map(|contract| Requirement {
                    id: contract.id.clone(),
                    fingerprint: contract.fingerprint.clone(),
                })
                .collect();
            let mut roots =
                operations.iter().flat_map(|operation| [operation.request.as_str(), operation.response.as_str()]).collect::<Vec<_>>();
            roots.extend(["harmos.model.v1.Position", "dokime.model.v1.Suites"]);
            roots.extend(
                host_contracts
                    .iter()
                    .filter_map(|contract| contract.protobuf.as_ref())
                    .flat_map(|protobuf| [protobuf.request.as_str(), protobuf.response.as_str()]),
            );
            let types = schema::operation_types(&types, &roots)?;
            let mut catalog = Declarations {
                operations,
                requires,
                types: Some(types),
                streams: vec![StreamDeclaration {
                    id: "probe.samples".into(),
                    row: "harmos.model.v1.Position".into(),
                    ..Default::default()
                }],
                sections: vec![Section {
                    r#type: "dokime.model.v1.Suites".into(),
                    payload: suites.encode_to_vec(),
                }],
                ..Default::default()
            };
            catalog.digest = schema::declarations_digest(&catalog);
            let file = Arc::new(Mutex::new(None));
            let observations = Arc::new(Mutex::new(Vec::new()));
            let provider = ScopeProbe {
                description: InstanceGetResponse {
                    id: "scope_fixture".into(),
                    version: "0.1.0".into(),
                    sdk: "rust".into(),
                    sdk_version: env!("CARGO_PKG_VERSION").into(),
                    message_bytes: support::settings().placement.message_bytes.get() as u64,
                    revision: 1,
                    declarations: Some(catalog),
                    started_at: Some(Timestamp::default()),
                },
                store,
                file: file.clone(),
                observations: observations.clone(),
            };
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let url = format!("http://{}", listener.local_addr()?);
            let (stop, stopped) = oneshot::channel();
            let provider = tokio::spawn(async move {
                Server::builder()
                    .add_service(InstanceServer::new(provider))
                    .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                        let _ = stopped.await;
                    })
                    .await
            });

            // Admit the Plugin and Run Its Scope Probes on the Real Child Carrier
            let created = client
                .plugins
                .create(
                    "scope_fixture",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url })),
                    },
                    PluginValue { enabled: true },
                )
                .await?;
            support::plugin_state(&client.plugins, created.entry.unwrap(), PluginState::Ready).await?;
            let cases = support::materialize(&client, "scope_fixture", "scope_fixture.capture", Default::default()).await?;
            let origins = cases.into_iter().map(|case| case.origin.unwrap()).collect();
            let cycle = client
                .cycles
                .create(
                    origins,
                    Metadata {
                        label: "Scope Probe".into(),
                        ..Default::default()
                    },
                    false,
                    String::new(),
                )
                .await?
                .cycle
                .unwrap();
            let entries =
                client.executions.list(cycle.cycle_id, Default::default(), Default::default(), Default::default()).await?.executions;
            let executions = entries.into_iter().map(|entry| entry.summary.unwrap().execution_id).collect::<Vec<_>>();
            let reference = support::native_file(&client, "scope-probe-file", &executions[0], &[1, 2, 3]).await?;
            *file.lock().unwrap() = Some(reference);
            client.executions.start(executions[0].clone(), ExecutionMode::Automatic).await?;
            let mut watch = client.executions.progress.watch(executions[0].clone()).await?;
            while watch.next().await?.is_some() {}
            let completed = client.executions.get(executions[0].clone()).await?.execution.unwrap();
            assert!(
                matches!(completed.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()),
                Some(execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == Termination::Completed as i32),
                "{:?}",
                completed.outcome
            );
            let results = observations.lock().unwrap().clone();
            assert_eq!(results.len(), 18, "{results:?}");
            for (operation, expected_refusal, refused) in results {
                assert_eq!(refused, expected_refusal, "{operation}: Expected Refusal {expected_refusal}");
            }
            process.close().await?;
            stop.send(()).map_err(|_| "Stop Scope Probe: Listener Already Closed")?;
            provider.await??;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn execution_isolation() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Start Three Executions Before Releasing Any Measurement
            let root = support::tempdir()?;
            let host = Host::open(root.path(), support::settings(), support::service_settings()).await?;
            let entry = host.add_managed("execution", &support::execution_fixture()?, &[]).await?;
            let first = host.materialize(&entry, "execution.capture", support::parameters(11.0, 0, true)).await?;
            let second = host.materialize(&entry, "execution.capture", support::parameters(22.0, 0, true)).await?;
            let third = host.materialize(&entry, "execution.capture", support::parameters(33.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![first, second]).await?;
            let other = host.start_cycle(vec![third]).await?;
            let executions = [&cycle.executions[0], &cycle.executions[1], &other.executions[0]];
            let mut prompts = Vec::new();
            for execution in executions {
                prompts.push(host.next_prompt(execution).await?);
            }
            assert_ne!(cycle.identifier, other.identifier);
            for (execution, prompt) in executions.into_iter().zip(prompts) {
                support::answer(&host, execution, prompt)?;
            }

            // Inspect Isolated Evidence and Execution Resources After Cleanup
            let mut snapshots = cycle.wait().await?.executions;
            snapshots.extend(other.wait().await?.executions);
            for (snapshot, value) in snapshots.iter().zip([11.0, 22.0, 33.0]) {
                assert_eq!(
                    snapshot.execution.termination,
                    Termination::Completed as i32,
                    "Complete Isolated Execution: {:?}",
                    snapshot
                        .evidence
                        .iter()
                        .filter(|evidence| evidence.kind.ends_with(":failure"))
                        .map(|evidence| (&evidence.kind, String::from_utf8_lossy(&evidence.payload)))
                        .collect::<Vec<_>>()
                );
                assert!(matches!(snapshot.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()),
                        Some(execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == ResultStatus::Passed as i32));
                assert_eq!(
                    snapshot.logs()?.iter().map(|log| log.message.parse::<f64>()).collect::<Result<Vec<_>, _>>()?,
                    vec![value, value]
                );
                assert!(!snapshot.samples.is_empty());
                for sample in &snapshot.samples {
                    assert_eq!(support::Reading::decode(sample.row.as_slice())?.value, value);
                }
                let records = snapshot.records("execution.capture.attempt");
                assert_eq!(records.len(), 1);
                let record = RecordsAppendRequest::decode(records[0].payload.as_slice())?;
                let attempt = support::Attempt::decode(record.payload.as_slice())?;
                assert_eq!(attempt.value, value);
                assert!(attempt.scope_refused);
                assert!(host.cancel(&snapshot.execution.identifier).is_err());
            }
            host.shutdown().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn concurrent_execution_clocks_advance_independently() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Admit Two Units on One Managed Connection
            let root = support::tempdir()?;
            let host = Host::open(root.path(), support::settings(), support::service_settings()).await?;
            let entry = host.add_managed("execution", &support::execution_fixture()?, &[]).await?;
            let first = host.materialize(&entry, "execution.capture", support::parameters(11.0, 0, true)).await?;
            let second = host.materialize(&entry, "execution.capture", support::parameters(22.0, 0, true)).await?;
            let left = host.start_cycle(vec![first]).await?;
            let right = host.start_cycle(vec![second]).await?;
            let left_id = left.executions[0].clone();
            let right_id = right.executions[0].clone();
            let left_prompt = host.next_prompt(&left_id).await?;
            let right_prompt = host.next_prompt(&right_id).await?;
            let held = host.execution(&left_id)?.samples;

            // Settle Only Right and Observe That Left's Timeline Stays Still
            support::answer(&host, &right_id, right_prompt)?;
            let right = right.wait().await?.executions.remove(0);
            assert_eq!(host.execution(&left_id)?.samples, held);
            assert_eq!(
                right.execution.termination,
                Termination::Completed as i32,
                "Complete Independent Execution Clock: {:?}",
                right
                    .evidence
                    .iter()
                    .filter(|evidence| evidence.kind.ends_with(":failure"))
                    .map(|evidence| (&evidence.kind, String::from_utf8_lossy(&evidence.payload)))
                    .collect::<Vec<_>>()
            );

            // Advance Left Through Its Own Sleep and Retain Separate Evidence
            support::answer(&host, &left_id, left_prompt)?;
            let left = left.wait().await?.executions.remove(0);
            assert_eq!(left.execution.termination, Termination::Completed as i32);
            for (snapshot, expected) in [(left, 11.0), (right, 22.0)] {
                assert_eq!(
                    snapshot.samples.iter().map(|sample| sample.source_ns).collect::<Vec<_>>(),
                    vec![1_000_000_000, 2_000_000_000, 3_000_000_000]
                );
                for sample in snapshot.samples {
                    assert_eq!(support::Reading::decode(sample.row.as_slice())?.value, expected);
                }
            }
            host.shutdown().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
