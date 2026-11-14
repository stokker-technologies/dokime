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
use std::{error::Error as StdError, vec::IntoIter};

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{encoding::Message, v1};
use dokime_storage::{Execution, Sample, Store};
use harmos_protocol::model as h;
use harmos_protocol::plugin::{
    InstanceGetRequest, InstanceGetResponse, InstanceInvokeRequest, InstanceInvokeResponse, InstanceSamplesRequest,
    InstanceSamplesResponse, InstanceStopRequest, InstanceStopResponse,
};
use tokio_stream::Iter;
use tonic::{Request, Response, Status, Streaming};

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
impl harmos_protocol::plugin::instance_server::Instance for AnalysisProvider {
    async fn get(&self, _: Request<InstanceGetRequest>) -> Result<Response<InstanceGetResponse>, Status> {
        Ok(Response::new(self.description.clone()))
    }
    type InvokeStream = Iter<IntoIter<Result<InstanceInvokeResponse, Status>>>;
    async fn invoke(&self, request: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        let mut input = request.into_inner();
        let Some(harmos_protocol::plugin::instance_invoke_request::Frame::Request(request)) =
            input.message().await?.and_then(|frame| frame.frame)
        else {
            return Err(Status::invalid_argument("Supply One Unary Request"));
        };
        assert!(request.scope.is_empty());
        let request = dokime_protocol::plugin::AnalysesComputeRequest::decode(request.payload.as_slice())
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        let factor = match request.parameters.unwrap().values["factor"].kind {
            Some(model::value::Kind::Number(value)) => value,
            _ => panic!("Expected a Numeric Factor"),
        };
        let samples = request.samples.unwrap();
        let output = dokime_protocol::model::AnalysisValues {
            key: "scaled".into(),
            axis_unit: "s".into(),
            axis: samples.axis,
            values: samples.values.into_iter().map(|value| value * factor).collect(),
            ..Default::default()
        };
        let result = dokime_protocol::plugin::AnalysesComputeResponse {
            outputs: if factor == 3.0 { vec![output.clone(), output] } else { vec![output] },
        };
        let response = harmos_protocol::plugin::Response {
            result: Some(harmos_protocol::plugin::response::Result::Payload(result.encode_to_vec())),
        };
        Ok(Response::new(tokio_stream::iter(vec![Ok(InstanceInvokeResponse {
            frame: Some(harmos_protocol::plugin::instance_invoke_response::Frame::Response(response)),
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

#[tokio::test]
async fn client_queries_advanced_plots() -> Result<(), Box<dyn StdError>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD,async {
        // Retain Correlated Streams and Numeric Group Labels
        let root=super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store=Store::open(root.path())?;
        let catalog=harmos_protocol::model::Declarations {types:Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),streams:["cells","calibration"].into_iter().map(|id|harmos_protocol::model::StreamDeclaration {id:id.into(),row:"dokime.host.v1.plots.PlotDomain".into(),..Default::default()}).collect(),..Default::default()};
        let mut catalog=catalog;
        let records=dokime_protocol::model::RecordContracts {records:vec![dokime_protocol::model::RecordContract {metadata:Some(dokime_protocol::model::Metadata {identifier:"event".into(),..Default::default()}),message:"dokime.host.v1.evidence.LogsAppendRequest".into()}]};
        catalog.sections.push(harmos_protocol::model::Section {r#type:"dokime.model.v1.RecordContracts".into(),payload:records.encode_to_vec()});
        store.create(&Execution {identifier:"advanced".into(),cycle:"advanced".into(),catalog:catalog.encode_to_vec(),origin:Some(dokime_protocol::model::Origin {kind:Some(model::origin::Kind::Import(dokime_protocol::model::Import::default()))}),..Default::default()})?;
        for stream in ["cells","calibration"] {
            store.open_stream("advanced",stream)?;
            let samples=(0..32).map(|index|Sample {stream:stream.into(),source_ns:index*1_000_000_000,row:dokime_protocol::host::plots::PlotDomain {min:if stream=="cells"{(index%4+1) as f64}else{2.0},max:(index+1) as f64}.encode_to_vec(),..Default::default()}).collect::<Vec<_>>();
            store.sample_batch("advanced",&samples)?;
        }
        for time in [5,25] {store.append("advanced",dokime_storage::Evidence {kind:"record:event".into(),source_ns:time*1_000_000_000,payload:dokime_protocol::host::evidence::RecordsAppendRequest {execution:"advanced".into(),kind:"event".into(),payload:dokime_protocol::host::evidence::LogsAppendRequest {message:"Captured Event".into(),..Default::default()}.encode_to_vec(),..Default::default()}.encode_to_vec(),..Default::default()})?;}
        store.close_streams("advanced")?;
        store.close("advanced",dokime_protocol::model::Termination::Completed,dokime_protocol::host::ExecutionOutcome::default())?;
        store.seal("advanced")?;
        let process=HostProcess::start_with_environment(root.path(), &credentials.environment).await?;

        let scope=Some(dokime_protocol::model::EvidenceScope {execution_id:Some("advanced".into()),..Default::default()});

        // Preserve Raw Samples or Select Bounded LTTB Vertices Without Truncation
        for (technique, limit, expected) in [(dokime_protocol::model::Downsample::Raw, 64, 32), (dokime_protocol::model::Downsample::Lttb, 8, 8)] {
            let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),y_keys:vec!["max".into()],technique:technique as i32,max_points:Some(limit),..Default::default()};
            let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
            let column=&reply.envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap().series[0];
            assert_eq!(column.y.len(),expected);
            assert_eq!(column.y.first().unwrap().value,Some(1.0));
            assert_eq!(column.y.last().unwrap().value,Some(32.0));
            assert!(column.instants.windows(2).all(|pair|pair[0]<pair[1]));
        }
        let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),y_keys:vec!["max".into()],technique:v1::Downsample::Raw as i32,max_points:Some(8),..Default::default()};
        assert_eq!(dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await.unwrap_err().code(),tonic::Code::FailedPrecondition);

        // Page Complete Plot Envelopes Without Losing Rows or Truncating Vertices
        let budget_request=dokime_protocol::host::plots::PlotRequest {plot_id:Some("p".repeat(418_500)),stream:"cells".into(),scope:scope.clone(),y_keys:vec!["max".into()],max_points:Some(128),..Default::default()};
        let mut snapshots=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?;
        let single=snapshots.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![budget_request.clone()], ..Default::default()}).await?.into_inner();
        let original_points=single.envelopes[0].diagnostics.as_ref().unwrap().point_count;
        let requests=vec![budget_request;10];
        let mut batch=snapshots.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:requests.clone(), ..Default::default()}).await?.into_inner();
        assert!(batch.envelopes.len()<requests.len());
        let cut=batch.resolved_at.clone();
        let mut envelopes=Vec::new();
        loop {
            envelopes.extend(batch.envelopes);
            if batch.next_page_token.is_empty() { break; }
            batch=snapshots.list(dokime_protocol::host::plots::SnapshotsListRequest {
                requests:requests.clone(),at:cut.clone(),page:Some(h::PageRequest {page_token:batch.next_page_token,..Default::default()})
            }).await?.into_inner();
            assert_eq!(batch.resolved_at,cut);
        }
        assert_eq!(envelopes.len(),requests.len());
        assert!(envelopes.iter().all(|envelope|envelope.diagnostics.as_ref().unwrap().point_count==original_points));
        assert!(envelopes.iter().all(|envelope|envelope.payload.as_ref().unwrap().frame.as_ref().unwrap().series[0].y.last().unwrap().value==Some(32.0)));

        // Invoke a Declared Unary Analysis with Bounded Raw Input and Typed Defaults
        let definition=dokime_protocol::model::Analysis {metadata:Some(dokime_protocol::model::Metadata {identifier:"analysis.scale".into(),..Default::default()}),version:1,max_samples:16,max_output_values:16,parameters:vec![dokime_protocol::model::Parameter {metadata:Some(dokime_protocol::model::Metadata {identifier:"factor".into(),..Default::default()}),r#type:v1::ValueType::Number as i32,min:Some(0.0),max:Some(4.0),expression:Some(dokime_protocol::model::ParameterExpression {kind:Some(model::parameter_expression::Kind::Literal(dokime_protocol::model::Value {kind:Some(model::value::Kind::Number(2.0))}))}),..Default::default()}],outputs:vec![dokime_protocol::model::AnalysisOutput {metadata:Some(dokime_protocol::model::Metadata {identifier:"scaled".into(),..Default::default()}),axis:"time".into(),axis_unit:"s".into(),..Default::default()}]};
        let types=prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?;
        let mut operation=h::OperationContract {id:"analysis.scale".into(),request:"dokime.model.v1.AnalysisRequest".into(),response:"dokime.model.v1.AnalysisResponse".into(),visibility:h::Visibility::Protected as i32,policy:Some(h::OperationPolicy {timeout:Some(prost_types::Duration::try_from(super::support::settings().placement.connect_timeout)?),..Default::default()}),..Default::default()};
        operation.fingerprint=harmos_protocol::schema::fingerprint(&operation.id,&types,&operation.request,&operation.response)?;
        let mut analysis_catalog=h::Declarations {types:Some(types),operations:vec![operation],sections:vec![h::Section {r#type:"dokime.model.v1.Suites".into(),payload:v1::Suites::default().encode_to_vec()},h::Section {r#type:"dokime.model.v1.Analyses".into(),payload:v1::Analyses {analyses:vec![definition]}.encode_to_vec()}],..Default::default()};
        analysis_catalog.digest=harmos_protocol::schema::declarations_digest(&analysis_catalog);
        let provider=AnalysisProvider {description:InstanceGetResponse {id:"analysis_fixture".into(),started_at:Some(prost_types::Timestamp::default()),revision:1,catalog:Some(analysis_catalog),..Default::default()}};
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let endpoint=format!("http://{}",listener.local_addr()?);
        let (stop,stopped)=tokio::sync::oneshot::channel::<()>();
        let provider_task=tokio::spawn(async move{tonic::transport::Server::builder().add_service(harmos_protocol::plugin::instance_server::InstanceServer::new(provider)).serve_with_incoming_shutdown(tokio_stream::wrappers::TcpListenerStream::new(listener),async {let _=stopped.await;}).await});
        let channel=tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client=dokime::client::HostClient::new(channel, super::support::client_settings())?;
        client.plugins.add(endpoint).await?;
        assert_eq!(dokime_protocol::host::analyses_client::AnalysesClient::connect(process.url.clone()).await?.list(dokime_protocol::host::AnalysesListRequest::default()).await?.into_inner().definitions.len(),1);
        let analysis_request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::TimeSeries as i32,y_keys:vec!["max".into()],viewport:Some(dokime_protocol::host::plots::PlotWindow {from:0.0,to:15.0}),analysis:Some(dokime_protocol::model::AnalysisSelection {id:"analysis.scale".into(),version:1,parameters:None}),max_points:Some(5),..Default::default()};
        let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![analysis_request.clone()], ..Default::default()}).await?.into_inner();
        let computed=reply.envelopes[0].payload.as_ref().unwrap().computed.as_ref().unwrap();
        assert_eq!(computed.series[0].input_samples,16);
        assert_eq!(computed.series[0].outputs[0].values,(1..=16).map(|value|value as f64*2.0).collect::<Vec<_>>());
        assert_eq!(computed.series[0].outputs[0].display_indices,vec![0,3,7,11,15]);
        let mut oversized=analysis_request.clone();oversized.viewport=None;
        assert!(dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![oversized], ..Default::default()}).await.is_err());
        let mut malformed=analysis_request;malformed.analysis.as_mut().unwrap().parameters=Some(dokime_protocol::model::MapValue {values:std::collections::HashMap::from([("factor".into(),dokime_protocol::model::Value {kind:Some(model::value::Kind::Number(3.0))})])});
        assert_eq!(dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![malformed], ..Default::default()}).await.unwrap_err().code(),tonic::Code::InvalidArgument);
        client.close();


        // Filter Stored Units Before Scaling the Selected Value
        let scaled=dokime_protocol::model::DataExpression {label:"Adjusted".into(),kind:Some(model::data_expression::Kind::Transform(Box::new(dokime_protocol::model::TransformExpression {input:Some(Box::new(dokime_protocol::model::FieldReference::new("cells").field("max"))),kind:Some(model::transform_expression::Kind::Scale(2.0))})))};
        let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::TimeSeries as i32,y_expressions:vec![scaled],filters:vec![dokime_protocol::model::PlotFilter {expression:Some(dokime_protocol::model::FieldReference::new("cells").field("max")),operator:v1::FilterOperator::Ge as i32,value:Some(dokime_protocol::model::Value {kind:Some(model::value::Kind::Number(17.0))})}],max_points:Some(128),..Default::default()};
        let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
        let payload=reply.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows,16);
        assert_eq!(payload.series[0].label,"Adjusted");
        assert!(payload.frame.as_ref().unwrap().series[0].y.iter().all(|value|value.value.unwrap()>=34.0));

        // Refuse Aggregate Variants That Have No Connected Renderer Instead of Returning Raw Values
        let unsupported=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::TimeSeries as i32,y_expressions:vec![dokime_protocol::model::DataExpression {kind:Some(model::data_expression::Kind::Aggregate(Box::new(dokime_protocol::model::AggregateExpression {input:Some(Box::new(dokime_protocol::model::FieldReference::new("cells").field("max"))),kind:v1::Aggregate::Mean as i32,..Default::default()}))),..Default::default()}],..Default::default()};
        assert_eq!(dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![unsupported], ..Default::default()}).await.unwrap_err().code(),tonic::Code::InvalidArgument);

        // Separate Summary and Viewport Windows with Bounded Event Markers
        let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::TimeSeries as i32,y_keys:vec!["max".into()],viewport:Some(dokime_protocol::host::plots::PlotWindow {from:16.0,to:31.0}),summary_window:Some(dokime_protocol::host::plots::PlotWindow {from:0.0,to:7.0}),marker_record_types:vec!["event".into()],max_points:Some(128),..Default::default()};
        let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
        let payload=reply.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(payload.summaries[0].count,8);
        assert_eq!(payload.summaries[0].mean.as_ref().and_then(|value| value.kind.as_ref()),Some(&model::value::Kind::Number(4.5)));
        assert_eq!(payload.markers.len(),1);
        assert_eq!(payload.markers[0].x.as_ref().unwrap().kind,Some(model::value::Kind::Number(25.0)));

        // Join a Coordinate from the Latest Earlier Source-Time Row
        let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::Xy as i32,y_keys:vec!["max".into()],x_expression:Some(dokime_protocol::model::FieldReference::new("calibration").field("min")),max_points:Some(128),..Default::default()};
        let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
        assert!(reply.envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap().series[0].x.iter().all(|value|value.kind==Some(model::value::Kind::Number(2.0))));

        // Reduce Time Series, Histograms and Boxes Independently for Each Group
        for kind in [dokime_protocol::model::PlotKind::TimeSeries,dokime_protocol::model::PlotKind::Histogram,dokime_protocol::model::PlotKind::BoxPlot] {
            let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:kind as i32,y_keys:vec!["max".into()],series_by:Some("min".into()),max_points:Some(128),..Default::default()};
            let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
            assert_eq!(reply.envelopes[0].payload.as_ref().unwrap().series.len(),4);
            assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows,32);
        }

        // Compute Backend Sigma Intervals and Logarithmic Contour Bins
        let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::Xy as i32,y_keys:vec!["max".into()],x_key:Some("min".into()),xy_interval:Some(dokime_protocol::model::XyInterval::Sigma as i32),interval:Some(dokime_protocol::model::PlotIntervalOptions {bin_width:1.0,sigma:2.0,clip_to_observed:false}),max_points:Some(128),..Default::default()};
        let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
        assert!(matches!(reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref(),Some(dokime_protocol::host::plots::plot_analysis_result::Kind::SigmaInterval(interval)) if interval.series[0].bins.len()==4 && interval.series[0].bins[0].count==8));
        let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope,plot_type:v1::PlotKind::Contour as i32,y_keys:vec!["max".into()],x_key:Some("min".into()),z_field:Some("max".into()),x_scale:v1::AxisScale::Log as i32,y_scale:v1::AxisScale::Log as i32,max_points:Some(128),..Default::default()};
        let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
        let Some(dokime_protocol::host::plots::plot_analysis_result::Kind::Contour(grid))=reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref() else {panic!("Expected Contour");};
        assert!(grid.x_edges.windows(3).all(|edges|((edges[1]/edges[0])-(edges[2]/edges[1])).abs()<1e-10));
        // Retain Authored Grid Highlights with the Selected Frame
        let literal=|value:f64|dokime_protocol::model::DataExpression {kind:Some(model::data_expression::Kind::Literal(dokime_protocol::model::Value {kind:Some(model::value::Kind::Number(value))})),..Default::default()};
        let request=dokime_protocol::host::plots::PlotRequest {stream:"cells".into(),scope:Some(dokime_protocol::model::EvidenceScope {execution_id:Some("advanced".into()),..Default::default()}),plot_type:v1::PlotKind::GridChart as i32,y_keys:vec!["max".into()],max_points:Some(128),reduction:Some(dokime_protocol::model::Reduction::Mean as i32),grid:Some(dokime_protocol::model::GridDefinition {row:Some(dokime_protocol::model::FieldReference::new("cells").field("min")),column:Some(literal(1.0)),frame:Some(literal(1.0)),highlights:vec![dokime_protocol::model::GridHighlight {row:Some(literal(1.0)),column:Some(literal(1.0)),label:"Selected".into()}],..Default::default()}),..Default::default()};
        let reply=dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone()).await?.list(dokime_protocol::host::plots::SnapshotsListRequest {requests:vec![request], ..Default::default()}).await?.into_inner();
        assert!(matches!(reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref(),Some(dokime_protocol::host::plots::plot_analysis_result::Kind::GridChart(grid)) if grid.frames[0].highlights.len()==1));
        process.close().await?;
        let _=stop.send(());provider_task.await??;
        Ok::<_,Box<dyn StdError>>(())
    }).await?
}
