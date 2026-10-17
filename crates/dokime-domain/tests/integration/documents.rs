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

// Local Imports
use dokime_domain::model;
use model::document::{Plot, PlotDefinition, format, validate_plot_definition};
use model::expressions::telemetry;

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

// A histogram bins the one value it declares on x; the host counts it on y.
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
        assert!(
            message.contains("histogram plots must declare exactly one x field or x expression"),
            "{message}"
        );
    }

    let mut clock = Plot::histogram("Voltage Distribution");
    clock.x(reconnection("voltage_v"));
    clock.x_format(format::clock("%H:%M:%S"));
    let message = refusal(&clock);
    assert!(
        message.contains("`x_format` on a histogram plot"),
        "{message}"
    );
}

// A box summary keeps its value on y, and its refusal names the plot and the axis.
#[test]
fn box_keeps_y() {
    let mut plot = Plot::box_summary("Voltage By Phase");
    plot.y(reconnection("voltage_v"));
    validate_plot_definition(&plot).unwrap();

    let message = refusal(&Plot::box_summary("Voltage By Phase"));
    assert!(
        message.contains("box_summary plots must declare exactly one y field or y expression"),
        "{message}"
    );
}
