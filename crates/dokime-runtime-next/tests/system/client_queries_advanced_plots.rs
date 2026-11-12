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
use dokime::{encoding::Message, v1};
use dokime_storage::{Execution, Sample, Store};

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
async fn client_queries_advanced_plots() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD,async {
        // Retain Correlated Streams and Numeric Group Labels
        let root=tempfile::tempdir()?;
        let store=Store::open(root.path())?;
        let catalog=harmos_protocol::v1::Catalog {types:Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),streams:["cells","calibration"].into_iter().map(|id|harmos_protocol::v1::StreamDeclaration {id:id.into(),row:"dokime.v1.PlotDomain".into(),..Default::default()}).collect(),..Default::default()};
        let mut catalog=catalog;
        let records=v1::RecordContracts {records:vec![v1::RecordContract {metadata:Some(v1::Metadata {identifier:"event".into(),..Default::default()}),message:"dokime.v1.WriteLog".into()}]};
        catalog.sections.push(harmos_protocol::v1::Section {r#type:"dokime.v1.RecordContracts".into(),payload:records.encode_to_vec()});
        store.create(&Execution {identifier:"advanced".into(),cycle:"advanced".into(),catalog:catalog.encode_to_vec(),origin:Some(v1::Origin {kind:Some(v1::origin::Kind::Import(v1::Import::default()))}),..Default::default()})?;
        for stream in ["cells","calibration"] {
            store.open_stream("advanced",stream)?;
            let samples=(0..32).map(|index|Sample {stream:stream.into(),source_ns:index*1_000_000_000,row:v1::PlotDomain {min:if stream=="cells"{(index%4+1) as f64}else{2.0},max:(index+1) as f64}.encode_to_vec(),..Default::default()}).collect::<Vec<_>>();
            store.sample_batch("advanced",&samples)?;
        }
        for time in [5,25] {store.append("advanced",dokime_storage::Evidence {kind:"record:event".into(),source_ns:time*1_000_000_000,payload:v1::PublishRecord {execution:"advanced".into(),kind:"event".into(),payload:v1::WriteLog {message:"Captured Event".into(),..Default::default()}.encode_to_vec(),..Default::default()}.encode_to_vec(),..Default::default()})?;}
        store.close_streams("advanced")?;
        store.close("advanced",v1::Termination::Completed,v1::ExecutionOutcome::default())?;
        store.seal("advanced")?;
        let process=HostProcess::start(root.path()).await?;
        let channel=tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut plots=v1::plots_client::PlotsClient::new(channel);
        let scope=Some(v1::EvidenceScope {execution_id:Some("advanced".into()),..Default::default()});

        // Filter Stored Units Before Scaling the Selected Value
        let scaled=v1::DataExpression {label:"Adjusted".into(),kind:Some(v1::data_expression::Kind::Transform(Box::new(v1::TransformExpression {input:Some(Box::new(v1::FieldReference::new("cells").field("max"))),kind:Some(v1::transform_expression::Kind::Scale(2.0))})))};
        let request=v1::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::TimeSeries as i32,y_expressions:vec![scaled],filters:vec![v1::PlotFilter {expression:Some(v1::FieldReference::new("cells").field("max")),operator:v1::FilterOperator::Ge as i32,value:Some(v1::Value {kind:Some(v1::value::Kind::Number(17.0))})}],max_points:Some(128),..Default::default()};
        let reply=plots.query_snapshots(v1::PlotsQuerySnapshotsRequest {requests:vec![request]}).await?.into_inner();
        let payload=reply.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows,16);
        assert_eq!(payload.series[0].label,"Adjusted");
        assert!(payload.frame.as_ref().unwrap().series[0].y.iter().all(|value|value.value.unwrap()>=34.0));

        // Separate Summary and Viewport Windows with Bounded Event Markers
        let request=v1::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::TimeSeries as i32,y_keys:vec!["max".into()],viewport:Some(v1::PlotWindow {from:16.0,to:31.0}),summary_window:Some(v1::PlotWindow {from:0.0,to:7.0}),marker_record_types:vec!["event".into()],max_points:Some(128),..Default::default()};
        let reply=plots.query_snapshots(v1::PlotsQuerySnapshotsRequest {requests:vec![request]}).await?.into_inner();
        let payload=reply.envelopes[0].payload.as_ref().unwrap();
        assert_eq!(payload.summaries[0].count,8);
        assert_eq!(payload.summaries[0].mean,Some(4.5));
        assert_eq!(payload.markers.len(),1);
        assert_eq!(payload.markers[0].x.as_ref().unwrap().kind,Some(v1::value::Kind::Number(25.0)));

        // Join a Coordinate from the Latest Earlier Source-Time Row
        let request=v1::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::Xy as i32,y_keys:vec!["max".into()],x_expression:Some(v1::FieldReference::new("calibration").field("min")),max_points:Some(128),..Default::default()};
        let reply=plots.query_snapshots(v1::PlotsQuerySnapshotsRequest {requests:vec![request]}).await?.into_inner();
        assert!(reply.envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap().series[0].x.iter().all(|value|value.kind==Some(v1::value::Kind::Number(2.0))));

        // Reduce Time Series, Histograms and Boxes Independently for Each Group
        for kind in [v1::PlotKind::TimeSeries,v1::PlotKind::Histogram,v1::PlotKind::BoxPlot] {
            let request=v1::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:kind as i32,y_keys:vec!["max".into()],series_by:Some("min".into()),max_points:Some(128),..Default::default()};
            let reply=plots.query_snapshots(v1::PlotsQuerySnapshotsRequest {requests:vec![request]}).await?.into_inner();
            assert_eq!(reply.envelopes[0].payload.as_ref().unwrap().series.len(),4);
            assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows,32);
        }

        // Compute Backend Sigma Intervals and Logarithmic Contour Bins
        let request=v1::PlotRequest {stream:"cells".into(),scope:scope.clone(),plot_type:v1::PlotKind::Xy as i32,y_keys:vec!["max".into()],x_key:Some("min".into()),xy_interval:Some(v1::XyInterval::Sigma as i32),interval:Some(v1::PlotIntervalOptions {bin_width:1.0,sigma:2.0,clip_to_observed:false}),max_points:Some(128),..Default::default()};
        let reply=plots.query_snapshots(v1::PlotsQuerySnapshotsRequest {requests:vec![request]}).await?.into_inner();
        assert!(matches!(reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref(),Some(v1::plot_analysis_result::Kind::SigmaInterval(interval)) if interval.series[0].bins.len()==4 && interval.series[0].bins[0].count==8));
        let request=v1::PlotRequest {stream:"cells".into(),scope,plot_type:v1::PlotKind::Contour as i32,y_keys:vec!["max".into()],x_key:Some("min".into()),z_field:Some("max".into()),x_scale:v1::AxisScale::Log as i32,y_scale:v1::AxisScale::Log as i32,max_points:Some(128),..Default::default()};
        let reply=plots.query_snapshots(v1::PlotsQuerySnapshotsRequest {requests:vec![request]}).await?.into_inner();
        let Some(v1::plot_analysis_result::Kind::Contour(grid))=reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref() else {panic!("Expected Contour");};
        assert!(grid.x_edges.windows(3).all(|edges|((edges[1]/edges[0])-(edges[2]/edges[1])).abs()<1e-10));
        // Retain Authored Grid Highlights with the Selected Frame
        let literal=|value:f64|v1::DataExpression {kind:Some(v1::data_expression::Kind::Literal(v1::Value {kind:Some(v1::value::Kind::Number(value))})),..Default::default()};
        let request=v1::PlotRequest {stream:"cells".into(),scope:Some(v1::EvidenceScope {execution_id:Some("advanced".into()),..Default::default()}),plot_type:v1::PlotKind::GridChart as i32,y_keys:vec!["max".into()],max_points:Some(128),reduction:Some(v1::Reduction::Mean as i32),grid:Some(v1::GridDefinition {row:Some(v1::FieldReference::new("cells").field("min")),column:Some(literal(1.0)),frame:Some(literal(1.0)),highlights:vec![v1::GridHighlight {row:Some(literal(1.0)),column:Some(literal(1.0)),label:"Selected".into()}],..Default::default()}),..Default::default()};
        let reply=plots.query_snapshots(v1::PlotsQuerySnapshotsRequest {requests:vec![request]}).await?.into_inner();
        assert!(matches!(reply.envelopes[0].payload.as_ref().unwrap().analysis.as_ref().unwrap().result.as_ref().unwrap().kind.as_ref(),Some(v1::plot_analysis_result::Kind::GridChart(grid)) if grid.frames[0].highlights.len()==1));
        process.close().await?;
        Ok::<_,Box<dyn std::error::Error>>(())
    }).await?
}
