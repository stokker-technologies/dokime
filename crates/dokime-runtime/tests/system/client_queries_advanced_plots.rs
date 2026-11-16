/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Queries Every Plot Kind

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::Aggregate;
use dokime::AggregateExpression;
use dokime::Analyses;
use dokime::Analysis;
use dokime::AnalysisOutput;
use dokime::AnalysisSelection;
use dokime::AnalysisValues;
use dokime::AxisScale;
use dokime::Client;
use dokime::DataExpression;
use dokime::Downsample;
use dokime::EvidenceScope;
use dokime::FieldReference;
use dokime::FilterOperator;
use dokime::GridDefinition;
use dokime::GridHighlight;
use dokime::Import;
use dokime::MapValue;
use dokime::Metadata;
use dokime::Origin;
use dokime::Parameter;
use dokime::ParameterExpression;
use dokime::PlotFilter;
use dokime::PlotIntervalOptions;
use dokime::PlotKind;
use dokime::RecordContract;
use dokime::RecordContracts;
use dokime::Reduction;
use dokime::Suites;
use dokime::Termination;
use dokime::TransformExpression;
use dokime::Value;
use dokime::ValueType;
use dokime::XyInterval;
use dokime::client::ExecutionOutcome;
use dokime::client::PlotsPlotDomain;
use dokime::client::PlotsPlotRequest;
use dokime::client::PlotsPlotWindow;
use dokime::client::PlotsSnapshotsListOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime::encoding::Message;
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::plots::plot_analysis_result::Kind as PlotAnalysisResultKind;
use dokime_protocol::model::data_expression::Kind as DataExpressionKind;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_protocol::model::parameter_expression::Kind as ParameterExpressionKind;
use dokime_protocol::model::transform_expression::Kind as TransformExpressionKind;
use dokime_protocol::model::value::Kind as ValueKind;
use dokime_protocol::plugin::AnalysesComputeRequest;
use dokime_protocol::plugin::AnalysesComputeResponse;
use dokime_storage::Evidence;
use dokime_storage::Execution;
use dokime_storage::Sample;
use dokime_storage::Store;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use harmos_protocol::model::Declarations;
use harmos_protocol::model::OperationContract;
use harmos_protocol::model::OperationPolicy;
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::Section;
use harmos_protocol::model::StreamDeclaration;
use harmos_protocol::model::Visibility;
use harmos_protocol::plugin::InstanceGetRequest;
use harmos_protocol::plugin::InstanceGetResponse;
use harmos_protocol::plugin::InstanceInvokeRequest;
use harmos_protocol::plugin::InstanceInvokeResponse;
use harmos_protocol::plugin::InstanceSamplesRequest;
use harmos_protocol::plugin::InstanceSamplesResponse;
use harmos_protocol::plugin::InstanceStopRequest;
use harmos_protocol::plugin::InstanceStopResponse;
use harmos_protocol::plugin::Response as InvocationResponse;
use harmos_protocol::plugin::instance_invoke_request::Frame as InvokeRequestFrame;
use harmos_protocol::plugin::instance_invoke_response::Frame as InvokeResponseFrame;
use harmos_protocol::plugin::instance_server::Instance;
use harmos_protocol::plugin::instance_server::InstanceServer;
use harmos_protocol::plugin::response::Result as InvocationResult;
use prost_types::Duration;
use prost_types::FileDescriptorSet;
use prost_types::Timestamp;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::vec::IntoIter;
use tokio::net::TcpListener;
use tokio_stream::Iter;
use tonic::Code;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tonic::Streaming;
use tonic::transport::Endpoint;
use tonic::transport::Server;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
struct AnalysisProvider {
    description: InstanceGetResponse,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Instance for AnalysisProvider {
    async fn get(&self, _: Request<InstanceGetRequest>) -> Result<Response<InstanceGetResponse>, Status> {
        Ok(Response::new(self.description.clone()))
    }
    type InvokeStream = Iter<IntoIter<Result<InstanceInvokeResponse, Status>>>;
    async fn invoke(&self, request: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        let mut input = request.into_inner();
        let Some(InvokeRequestFrame::Request(request)) = input.message().await?.and_then(|frame| frame.frame) else {
            return Err(Status::invalid_argument("Supply One Unary Request"));
        };
        assert!(request.scope.is_empty());
        let request =
            AnalysesComputeRequest::decode(request.payload.as_slice()).map_err(|error| Status::invalid_argument(error.to_string()))?;
        let factor = match request.parameters.unwrap().values["factor"].kind {
            Some(ValueKind::Number(value)) => value,
            _ => panic!("Expected a Numeric Factor"),
        };
        let samples = request.samples.unwrap();
        let output = AnalysisValues {
            key: "scaled".into(),
            axis_unit: "s".into(),
            axis: samples.axis,
            values: samples.values.into_iter().map(|value| value * factor).collect(),
            ..Default::default()
        };
        let result = AnalysesComputeResponse {
            outputs: if factor == 3.0 { vec![output.clone(), output] } else { vec![output] },
        };
        let response = InvocationResponse {
            result: Some(InvocationResult::Payload(result.encode_to_vec())),
        };
        Ok(Response::new(tokio_stream::iter(vec![Ok(InstanceInvokeResponse {
            frame: Some(InvokeResponseFrame::Response(response)),
        })])))
    }
    type SamplesStream = Iter<IntoIter<Result<InstanceSamplesResponse, Status>>>;
    async fn samples(&self, _: Request<Streaming<InstanceSamplesRequest>>) -> Result<Response<Self::SamplesStream>, Status> {
        Err(Status::unimplemented("This Fixture Has No Streams"))
    }
    async fn stop(&self, _: Request<InstanceStopRequest>) -> Result<Response<InstanceStopResponse>, Status> {
        Ok(Response::new(InstanceStopResponse {}))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn client_queries_advanced_plots() -> Result<(), Box<dyn StdError>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD,async {
        // Retain Correlated Streams and Numeric Group Labels
        let root=super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store=Store::open(root.path())?;
        let catalog=Declarations {types:Some(FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?),streams:["cells","calibration"].into_iter().map(|id|StreamDeclaration {id:id.into(),row:"dokime.host.v1.plots.PlotDomain".into(),..Default::default()}).collect(),..Default::default()};
        let mut catalog=catalog;
        let records=RecordContracts {records:vec![RecordContract {metadata:Some(Metadata {identifier:"event".into(),..Default::default()}),message:"dokime.host.v1.evidence.LogsAppendRequest".into()}]};
        catalog.sections.push(Section {r#type:"dokime.model.v1.RecordContracts".into(),payload:records.encode_to_vec()});
        store.create(&Execution {identifier:"advanced".into(),cycle:"advanced".into(),catalog:catalog.encode_to_vec(),origin:Some(Origin {kind:Some(OriginKind::Import(Import::default()))}),..Default::default()})?;
        for stream in ["cells","calibration"] {
            store.open_stream("advanced",stream)?;
            let samples=(0..32).map(|index|Sample {stream:stream.into(),source_ns:index*1_000_000_000,row:PlotsPlotDomain {min:if stream=="cells"{(index%4+1) as f64}else{2.0},max:(index+1) as f64}.encode_to_vec(),..Default::default()}).collect::<Vec<_>>();
            store.sample_batch("advanced",&samples)?;
        }
        for time in [5,25] {store.append("advanced",Evidence {kind:"record:event".into(),source_ns:time*1_000_000_000,payload:RecordsAppendRequest {execution:"advanced".into(),kind:"event".into(),payload:LogsAppendRequest {message:"Captured Event".into(),..Default::default()}.encode_to_vec(),..Default::default()}.encode_to_vec(),..Default::default()})?;
}
        store.close_streams("advanced")?;
        store.close("advanced",Termination::Completed,ExecutionOutcome::default())?;
        store.seal("advanced")?;
        let process=HostProcess::start_with_environment(root.path(), &credentials.environment).await?;

        let scope=Some(EvidenceScope {execution_id:Some("advanced".into()),..Default::default()});

        // Preserve Raw Samples or Select Bounded LTTB Vertices Without Truncation
        for (technique, limit, expected) in [(Downsample::Raw, 64, 32), (Downsample::Lttb, 8, 8)] {
            let request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),y_keys:vec!["max".into()],technique:technique as i32,max_points:Some(limit),..Default::default()};
            let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
            let column=&reply.envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap().series[0];
            assert_eq!(column.y.len(),expected);
            assert_eq!(column.y.first().unwrap().value,Some(1.0));
            assert_eq!(column.y.last().unwrap().value,Some(32.0));
            assert!(column.instants.windows(2).all(|pair|pair[0]<pair[1]));
        }
        let request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),y_keys:vec!["max".into()],technique:Downsample::Raw as i32,max_points:Some(8),..Default::default()};
        assert_eq!(Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await.unwrap_err().code(),Code::FailedPrecondition);

        // Page Complete Plot Envelopes Without Losing Rows or Truncating Vertices
        let budget_request=PlotsPlotRequest {plot_id:Some("p".repeat(418_500)),stream:"cells".into(),scope:scope.clone(),y_keys:vec!["max".into()],max_points:Some(128),..Default::default()};
        let snapshots =Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone();
        let single=snapshots.list(vec![budget_request.clone()]).await?;
        let original_points=single.envelopes[0].diagnostics.as_ref().unwrap().point_count;
        let requests=vec![budget_request;10];
        let mut batch=snapshots.list_with(requests.clone(), PlotsSnapshotsListOptions {page:Some(PageRequest {max_bytes:1_000_000,..Default::default()}),..Default::default()}).await?;
        assert!(batch.envelopes.len()<requests.len());
        let cut=batch.resolved_at.clone();
        let mut envelopes=Vec::new();
        loop {
            envelopes.extend(batch.envelopes);
            if batch.next_page_token.is_empty() { break;
 }
            batch=snapshots.list_with(requests.clone(), PlotsSnapshotsListOptions { at: cut.clone(), page: Some(PageRequest {page_token:batch.next_page_token,max_bytes:1_000_000,..Default::default()}), ..Default::default() }).await?;
            assert_eq!(batch.resolved_at,cut);
        }
        assert_eq!(envelopes.len(),requests.len());
        assert!(envelopes.iter().all(|envelope|envelope.diagnostics.as_ref().unwrap().point_count==original_points));
        assert!(envelopes.iter().all(|envelope|envelope.payload.as_ref().unwrap().frame.as_ref().unwrap().series[0].y.last().unwrap().value==Some(32.0)));

        // Invoke a Declared Unary Analysis with Bounded Raw Input and Typed Defaults
        let definition=Analysis {metadata:Some(Metadata {identifier:"analysis.scale".into(),..Default::default()}),version:1,max_samples:16,max_output_values:16,parameters:vec![Parameter {metadata:Some(Metadata {identifier:"factor".into(),..Default::default()}),r#type:ValueType::Number as i32,min:Some(0.0),max:Some(4.0),expression:Some(ParameterExpression {kind:Some(ParameterExpressionKind::Literal(Value {kind:Some(ValueKind::Number(2.0))}))}),..Default::default()}],outputs:vec![AnalysisOutput {metadata:Some(Metadata {identifier:"scaled".into(),..Default::default()}),axis:"time".into(),axis_unit:"s".into(),..Default::default()}]};
        let types=FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?;
        let mut operation=OperationContract {id:"analysis.scale".into(),request:"dokime.plugin.v1.AnalysesComputeRequest".into(),response:"dokime.plugin.v1.AnalysesComputeResponse".into(),visibility:Visibility::Protected as i32,policy:Some(OperationPolicy {timeout:Some(Duration::try_from(super::support::OPERATION_TIMEOUT)?),..Default::default()}),..Default::default()};
        operation.fingerprint=harmos_protocol::schema::fingerprint(&operation.id,&types,&operation.request,&operation.response)?;
        let types=harmos_protocol::schema::operation_types(&types,&[&operation.request,&operation.response,"dokime.model.v1.Suites","dokime.model.v1.Analyses"])?;
        let mut analysis_catalog=Declarations {types:Some(types),operations:vec![operation],sections:vec![Section {r#type:"dokime.model.v1.Suites".into(),payload:Suites::default().encode_to_vec()},Section {r#type:"dokime.model.v1.Analyses".into(),payload:Analyses {analyses:vec![definition]}.encode_to_vec()}],..Default::default()};
        analysis_catalog.digest=harmos_protocol::schema::declarations_digest(&analysis_catalog);
        let provider=AnalysisProvider {description:InstanceGetResponse {id:"analysis_fixture".into(),version:"0.1.0".into(),sdk:"rust".into(),sdk_version:env!("CARGO_PKG_VERSION").into(),message_bytes:super::support::settings().placement.message_bytes.get() as u64,started_at:Some(Timestamp::default()),revision:1,declarations:Some(analysis_catalog)}};
        let listener=TcpListener::bind("127.0.0.1:0").await?;
        let endpoint=format!("http://{}",listener.local_addr()?);
        let (stop,stopped)=tokio::sync::oneshot::channel::<()>();
        let provider_task=tokio::spawn(async move{Server::builder().add_service(InstanceServer::new(provider)).serve_with_incoming_shutdown(tokio_stream::wrappers::TcpListenerStream::new(listener),async {let _=stopped.await;
}).await});
        let channel=Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client=Client::from_channel(channel, super::support::client_settings())?;
        let entry = client.plugins.create("analysis_fixture", PluginPlacement { kind: Some(PluginPlacementKind::Remote(PluginRemote { url: endpoint })) }, PluginValue { enabled: true }).await?.entry.unwrap();
        super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
        assert_eq!(Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.analyses.clone().list().await?.definitions.len(),1);
        let analysis_request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:PlotKind::TimeSeries as i32,y_keys:vec!["max".into()],viewport:Some(PlotsPlotWindow {from:0.0,to:15.0}),analysis:Some(AnalysisSelection {id:"analysis.scale".into(),version:1,parameters:None}),max_points:Some(5),..Default::default()};
        let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![analysis_request.clone()]).await?;
        let computed=reply.envelopes[0].payload.as_ref().unwrap().computed.as_ref().unwrap();
        assert_eq!(computed.series[0].input_samples,16);
        assert_eq!(computed.series[0].outputs[0].values,(1..=16).map(|value|value as f64*2.0).collect::<Vec<_>>());
        assert_eq!(computed.series[0].outputs[0].display_indices,vec![0,3,7,11,15]);
        let mut oversized=analysis_request.clone();
oversized.viewport=None;
        assert!(Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![oversized]).await.is_err());
        let mut malformed=analysis_request;
malformed.analysis.as_mut().unwrap().parameters=Some(MapValue {values:HashMap::from([("factor".into(),Value {kind:Some(ValueKind::Number(3.0))})])});
        assert_eq!(Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![malformed]).await.unwrap_err().code(),Code::InvalidArgument);

        // Filter Stored Units Before Scaling the Selected Value
        let scaled=DataExpression {label:"Adjusted".into(),kind:Some(DataExpressionKind::Transform(Box::new(TransformExpression {input:Some(Box::new(FieldReference::new("cells").field("max"))),kind:Some(TransformExpressionKind::Scale(2.0))})))};
        let request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:PlotKind::TimeSeries as i32,y_expressions:vec![scaled],filters:vec![PlotFilter {expression:Some(FieldReference::new("cells").field("max")),operator:FilterOperator::Ge as i32,value:Some(Value {kind:Some(ValueKind::Number(17.0))})}],max_points:Some(128),..Default::default()};
        let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
        let payload=reply.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows,16);
        assert!(payload.frame.as_ref().unwrap().series[0].y.iter().all(|value|value.value.unwrap()>=34.0));

        // Refuse Aggregate Variants That Have No Connected Renderer Instead of Returning Raw Values
        let unsupported=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:PlotKind::TimeSeries as i32,y_expressions:vec![DataExpression {kind:Some(DataExpressionKind::Aggregate(Box::new(AggregateExpression {input:Some(Box::new(FieldReference::new("cells").field("max"))),kind:Aggregate::Mean as i32,..Default::default()}))),..Default::default()}],..Default::default()};
        assert_eq!(Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![unsupported]).await.unwrap_err().code(),Code::InvalidArgument);

        // Separate Summary and Viewport Windows with Bounded Event Markers
        let request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:PlotKind::TimeSeries as i32,y_keys:vec!["max".into()],viewport:Some(PlotsPlotWindow {from:16.0,to:31.0}),summary_window:Some(PlotsPlotWindow {from:0.0,to:7.0}),marker_record_types:vec!["event".into()],max_points:Some(128),..Default::default()};
        let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
        let payload=reply.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(payload.summaries[0].count,8);
        assert_eq!(payload.summaries[0].mean.as_ref().and_then(|value| value.kind.as_ref()),Some(&ValueKind::Number(4.5)));
        assert_eq!(payload.markers.len(),1);
        assert_eq!(payload.markers[0].x.as_ref().unwrap().kind,Some(ValueKind::Number(25.0)));

        // Join a Coordinate from the Latest Earlier Source-Time Row
        let request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:PlotKind::Xy as i32,y_keys:vec!["max".into()],x_expression:Some(FieldReference::new("calibration").field("min")),max_points:Some(128),..Default::default()};
        let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
        assert!(reply.envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap().series[0].x.iter().all(|value|value.kind==Some(ValueKind::Number(2.0))));

        // Reduce Time Series, Histograms and Boxes Independently for Each Group
        for kind in [PlotKind::TimeSeries,PlotKind::Histogram,PlotKind::BoxPlot] {
            let request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:kind as i32,y_keys:vec!["max".into()],series_by:Some("min".into()),max_points:Some(128),..Default::default()};
            let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
            assert_eq!(reply.envelopes[0].payload.as_ref().unwrap().series.len(),4);
            assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows,32);
        }

        // Compute Backend Sigma Intervals and Logarithmic Contour Bins
        let request=PlotsPlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:PlotKind::Xy as i32,y_keys:vec!["max".into()],x_key:Some("min".into()),xy_interval:Some(XyInterval::Sigma as i32),interval:Some(PlotIntervalOptions {bin_width:1.0,sigma:2.0,clip_to_observed:false}),max_points:Some(128),..Default::default()};
        let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
        assert!(matches!(reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref(),Some(PlotAnalysisResultKind::SigmaInterval(interval)) if interval.series[0].bins.len()==4 && interval.series[0].bins[0].count==8));
        let request=PlotsPlotRequest {stream:"cells".into(),scope,plot_type:PlotKind::Contour as i32,y_keys:vec!["max".into()],x_key:Some("min".into()),z_field:Some("max".into()),x_scale:AxisScale::Log as i32,y_scale:AxisScale::Log as i32,max_points:Some(128),..Default::default()};
        let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
        let Some(PlotAnalysisResultKind::Contour(grid))=reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref() else {panic!("Expected Contour");
};
        assert!(grid.x_edges.windows(3).all(|edges|((edges[1]/edges[0])-(edges[2]/edges[1])).abs()<1e-10));
        // Retain Authored Grid Highlights with the Selected Frame
        let literal=|value:f64|DataExpression {kind:Some(DataExpressionKind::Literal(Value {kind:Some(ValueKind::Number(value))})),..Default::default()};
        let request=PlotsPlotRequest {stream:"cells".into(),scope:Some(EvidenceScope {execution_id:Some("advanced".into()),..Default::default()}),plot_type:PlotKind::GridChart as i32,y_keys:vec!["max".into()],max_points:Some(128),reduction:Some(Reduction::Mean as i32),grid:Some(GridDefinition {row:Some(FieldReference::new("cells").field("min")),column:Some(literal(1.0)),frame:Some(literal(1.0)),highlights:vec![GridHighlight {row:Some(literal(1.0)),column:Some(literal(1.0)),label:"Selected".into()}],..Default::default()}),..Default::default()};
        let reply=Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.plots.snapshots.clone().list(vec![request]).await?;
        assert!(matches!(reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref(),Some(PlotAnalysisResultKind::GridChart(grid)) if grid.frames[0].highlights.len()==1));
        process.close().await?;
        let _=stop.send(());
provider_task.await??;
        Ok::<_,Box<dyn StdError>>(())
    }).await?
    }))
}
