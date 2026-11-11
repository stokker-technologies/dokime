/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A plot declaration is refused at authoring time when its axes cannot draw it.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use model::document::{Plot, PlotDefinition, format, validate_plot_definition};
use model::expressions::telemetry;

// Local Imports
use dokime_domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn reconnection(field: &str) -> model::expressions::DataExpression {
    telemetry("bench.reconnection", field)
}

fn refusal(plot: &PlotDefinition) -> String {
    validate_plot_definition(plot).expect_err("plot is refused")
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn xy_axes_and_styles() {
    use model::document::{PlotStyle, PlotType};

    let mut plot = Plot::xy("Tracking Path");
    assert_eq!(plot.plot_type, PlotType::Xy);
    assert_eq!(plot.style, PlotStyle::Line);
    assert!(validate_plot_definition(&plot).is_err());
    plot.x(telemetry("tracking", "easting_m"));
    assert!(validate_plot_definition(&plot).is_err());
    plot.y(telemetry("tracking", "northing_m"));
    for style in [PlotStyle::Line, PlotStyle::Points, PlotStyle::LinePoints, PlotStyle::Step] {
        plot.style(style);
        validate_plot_definition(&plot).unwrap();
        let encoded = serde_json::to_value(&plot).unwrap();
        assert_eq!(encoded["plot_type"], "xy");
        let decoded: PlotDefinition = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded, plot);
    }
}

#[dokime_tracing::trace_test]
#[test]
fn reference_values_validation() {
    use model::document::{DocumentDefinition, DocumentSection, ReferenceValue};
    use model::expressions::{detail, parameter};

    let mut plot = Plot::time_series("Temperature");
    plot.y(telemetry("temperature", "temperature_c"));
    plot.reference_band("Requirement", -10.0, 10.0);
    plot.reference_band("Limits", parameter("lower_limit_c"), parameter("upper_limit_c"));
    plot.reference_tolerance("Requirement", 0.0, parameter("angle_tolerance_arcsec"));
    plot.reference_tolerance("Target ± Band", parameter("target_c"), parameter("band_c"));
    plot.reference_line("Limit", parameter("limit_c"));
    plot.reference_line("Fixed", 5.0);
    plot.reference_line("Record", detail("reading", "temperature_c"));
    validate_plot_definition(&plot).unwrap();
    assert_eq!(plot.reference_bands[0].min, Some(ReferenceValue::Constant(-10.0)));
    assert_eq!(plot.reference_bands[1].min, Some(ReferenceValue::Expression(parameter("lower_limit_c"))));
    assert_eq!(plot.reference_bands[2].center, Some(ReferenceValue::Constant(0.0)));
    let encoded = serde_json::to_value(&plot).unwrap();
    let decoded: PlotDefinition = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, plot);

    for (min, max, tolerance) in [(10.0, -10.0, 1.0), (-10.0, 10.0, -1.0), (f64::NAN, 10.0, 1.0)] {
        let mut plot = Plot::time_series("Invalid Temperature");
        plot.y(telemetry("temperature", "temperature_c"));
        plot.reference_band("Bounds", min, max);
        plot.reference_tolerance("Tolerance", 0.0, tolerance);
        assert!(refusal(&plot).contains("Invalid Temperature"));
        let mut section = DocumentSection::new("Measurements");
        section.plot(plot);
        let mut document = DocumentDefinition::new("Temperature");
        document.section(section);
        assert!(model::document::validate_document_definition(&document, 8, 1000).is_err());
    }
}

#[dokime_tracing::trace_test]
#[test]
fn display_scale_validation() {
    let mut plot = Plot::time_series("Distance Delta");
    plot.y(telemetry("distance", "delta_m"));
    let unscaled = serde_json::to_value(&plot).unwrap();
    assert!(unscaled.get("scale").is_none());
    let decoded: PlotDefinition = serde_json::from_value(unscaled).unwrap();
    assert_eq!(decoded.scale, None);

    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        plot.scale(scale);
        let message = refusal(&plot);
        assert!(message.contains("Distance Delta"), "{message}");
        assert!(message.contains("positive finite"), "{message}");
        let mut section = model::document::DocumentSection::new("Measurements");
        section.plot(plot.clone());
        let mut document = model::document::DocumentDefinition::new("Distance");
        document.section(section);
        let issues = model::document::validate_document_definition(&document, 8, 1000).unwrap_err();
        assert!(issues.iter().any(|issue| issue.contains("Distance Delta")));
    }
    for scale in [1.0, 1e3, 1e-3] {
        plot.scale(scale);
        validate_plot_definition(&plot).unwrap();
        let encoded = serde_json::to_value(&plot).unwrap();
        let decoded: PlotDefinition = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded, plot);
    }
}

#[dokime_tracing::trace_test]
#[test]
fn display_scale_refuses_non_time_series_and_custom_axes() {
    use model::document::{PlotAxisKind, PlotType};

    for plot_type in [
        PlotType::Bar,
        PlotType::Pie,
        PlotType::Xy,
        PlotType::Trajectory,
        PlotType::Contour,
        PlotType::GridChart,
        PlotType::FrequencySeries,
        PlotType::Histogram,
        PlotType::BoxPlot,
    ] {
        let mut plot = PlotDefinition::with_type("Unsupported Axis", plot_type);
        if plot.plot_type == PlotType::Pie {
            plot.y(reconnection("voltage_v"));
        }
        plot.scale(1e3);
        let message = refusal(&plot);
        assert!(message.contains("Unsupported Axis"), "{message}");
        assert!(message.contains("time_series y"), "{message}");
    }

    let mut plot = Plot::time_series("Distance Delta");
    plot.y(telemetry("distance", "delta_m"));
    plot.scale(1e3);
    plot.x(telemetry("distance", "elapsed_s"));
    assert!(refusal(&plot).contains("Distance Delta"));
    plot.x_expression = None;
    plot.x_axis(PlotAxisKind::elapsed("s"));
    assert!(refusal(&plot).contains("other x axes"));
    plot.x_axis(PlotAxisKind::WallTime);
    plot.analysis = Some(model::analysis::Selection {
        id: "custom".to_owned(),
        version: 1,
        parameters: Default::default(),
    });
    assert!(refusal(&plot).contains("custom analyses"));
}

// A histogram bins the one value it declares on x; the host counts it on y.
#[dokime_tracing::trace_test]
#[test]
fn histogram_bins_x() {
    let mut plot = Plot::histogram("Time Until Ready");
    plot.unit("s");
    plot.x(reconnection("time_until_ready_s"));
    plot.x_format(format::fixed(1));
    validate_plot_definition(&plot).unwrap();

    let mut field = Plot::histogram("Time Until Ready");
    field.x = Some("time_until_ready_s".to_owned());
    validate_plot_definition(&field).unwrap();
}

// A y binding on a histogram is refused with where the value belongs instead.
#[dokime_tracing::trace_test]
#[test]
fn histogram_refuses_y() {
    let mut value_on_y = Plot::histogram("Voltage Distribution");
    value_on_y.y(reconnection("voltage_v"));
    let message = refusal(&value_on_y);
    assert!(message.contains("histogram"), "{message}");
    assert!(message.contains("x axis"), "{message}");
    assert!(message.contains(".x(…)"), "{message}");

    let mut both = Plot::histogram("Voltage Distribution");
    both.x(reconnection("voltage_v"));
    both.y(reconnection("current_a"));
    assert_eq!(refusal(&both), message);

    let mut count_format = Plot::histogram("Voltage Distribution");
    count_format.x(reconnection("voltage_v"));
    count_format.y_format(format::fixed(1));
    let message = refusal(&count_format);
    assert!(message.contains("histogram"), "{message}");
    assert!(message.contains("count"), "{message}");
}

// A histogram without exactly one value, or with a time format, says which axis.
#[dokime_tracing::trace_test]
#[test]
fn histogram_requires_x() {
    for plot in [
        Plot::histogram("Voltage Distribution"),
        {
            let mut timestamp = Plot::histogram("Voltage Distribution");
            timestamp.x = Some("timestamp".to_owned());
            timestamp
        },
        {
            let mut two = Plot::histogram("Voltage Distribution");
            two.x = Some("current_a".to_owned());
            two.x(reconnection("voltage_v"));
            two
        },
    ] {
        let message = refusal(&plot);
        assert!(message.contains("histogram plots must declare exactly one x field or x expression"), "{message}");
    }

    let mut clock = Plot::histogram("Voltage Distribution");
    clock.x(reconnection("voltage_v"));
    clock.x_format(format::clock("%H:%M:%S"));
    let message = refusal(&clock);
    assert!(message.contains("`x_format` on a histogram plot"), "{message}");
}

// A box plot requires exactly one value on y.
#[dokime_tracing::trace_test]
#[test]
fn box_plot_requires_one_y_binding() {
    let mut plot = Plot::box_plot("Voltage by Phase");
    plot.y(reconnection("voltage_v"));
    validate_plot_definition(&plot).unwrap();

    assert!(validate_plot_definition(&Plot::box_plot("Voltage by Phase")).is_err());
    plot.y(reconnection("current_a"));
    assert!(validate_plot_definition(&plot).is_err());
}

#[test]
fn catalog_options_and_retired_names() {
    use model::document::{BarArrangement, PlotType, XyInterval};
    for (name, replacement) in [
        ("xy_interval", "XY with interval=sigma"),
        ("heatmap", "CONTOUR"),
        ("xy_value_heatmap", "CONTOUR"),
        ("wifi_channel_occupancy", "GRID_CHART"),
        ("grouped_bar", "BAR with arrangement=grouped"),
        ("stacked_bar", "BAR with arrangement=stacked"),
        ("pareto", "BAR with pareto=true"),
        ("donut", "PIE with hole=0.5"),
        ("box_summary", "BOX_PLOT"),
        ("aggregate_table", "Summary card"),
    ] {
        let error = serde_json::from_value::<PlotType>(serde_json::json!(name)).unwrap_err();
        assert!(error.to_string().contains(replacement), "{error}");
    }
    let mut plot = PlotDefinition::with_type("Totals", PlotType::Bar);
    plot.x = Some("category".into());
    plot.y = vec!["value".into()];
    plot.arrangement = Some(BarArrangement::Grouped);
    assert!(validate_plot_definition(&plot).is_err());
    plot.series_by = Some("group".into());
    validate_plot_definition(&plot).unwrap();
    plot.arrangement = Some(BarArrangement::Stacked);
    validate_plot_definition(&plot).unwrap();
    plot.pareto = true;
    assert!(validate_plot_definition(&plot).is_err());
    plot.arrangement = None;
    assert!(validate_plot_definition(&plot).is_err());
    plot.series_by = None;
    validate_plot_definition(&plot).unwrap();
    plot.pareto = false;
    plot.plot_type = PlotType::Pie;
    for hole in [None, Some(0.0), Some(0.5)] {
        plot.hole = hole;
        validate_plot_definition(&plot).unwrap();
    }
    for hole in [-0.1, 1.0, f64::NAN, f64::INFINITY] {
        plot.hole = Some(hole);
        assert!(validate_plot_definition(&plot).is_err());
    }
    plot.hole = None;
    plot.plot_type = PlotType::Xy;
    plot.xy_interval = Some(XyInterval::Sigma);
    validate_plot_definition(&plot).unwrap();
    let encoded = serde_json::to_value(&plot).unwrap();
    let decoded = serde_json::from_value::<PlotDefinition>(encoded).unwrap();
    assert_eq!(decoded.xy_interval, Some(XyInterval::Sigma));
}
