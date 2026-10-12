# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Document plot authoring examples backed by the Rust document contract."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import json

# Third Party Imports

# Local Imports
from dokime import BarPlot
from dokime import BoxSummaryDeclaration
from dokime import Document
from dokime import FrequencySeriesPlot
from dokime import HistogramPlot
from dokime import Section
from dokime import SigmaInterval
from dokime import TimeSeriesPlot
from dokime import TrajectoryPlot
from dokime import XyIntervalPlot
from dokime import XyLinePlot
from dokime import XyScatterPlot
from dokime import dimension
from dokime import telemetry

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build_plot_catalog_document():
    timestamp = telemetry("tracking", "timestamp")
    elapsed = telemetry("tracking", "elapsed_s")
    distance = telemetry("tracking", "distance_m")
    position_x = telemetry("tracking", "position_x_m")
    position_y = telemetry("tracking", "position_y_m")
    reacquire = telemetry("tracking", "target_reacquire_time_s")
    loss_distance = telemetry("tracking", "connection_loss_distance_m")

    return Document("examples.document.plot_catalog", "Document Plot Catalog").section(
        Section("examples.document.plot_catalog.tracking", "Tracking")
        .plot(
            TimeSeriesPlot("Tracking Distance")
            .y(distance)
            .with_group("Time Series")
            .style("line_points")
            .marker_records(["target.lost", "target.acquired"])
        )
        .plot(
            XyScatterPlot("XY Position Samples")
            .x(position_x)
            .y(position_y)
            .with_group("Position")
        )
        .plot(
            XyLinePlot("Distance Over Elapsed Time")
            .x(elapsed)
            .y(distance)
            .with_group("Position")
        )
        .plot(
            XyIntervalPlot(
                "Distance Sigma Interval",
                interval=SigmaInterval(bin_width=1.0, sigma=3.0),
            )
            .x(elapsed)
            .y(distance)
            .with_group("Statistics")
        )
        .plot(
            TrajectoryPlot("Trajectory")
            .x(position_x)
            .y(position_y)
            .with_group("Position")
        )
        .plot(
            FrequencySeriesPlot(
                "Distance Frequency",
                fft={"sample_rate_hz": 10.0, "window": "hann", "detrend": "mean"},
            )
            .x(timestamp)
            .y(distance)
            .with_group("Frequency")
        )
        .plot(
            HistogramPlot("Target Reacquire Time", bins=16)
            .y(reacquire)
            .with_unit("s")
            .with_group("Stability")
        )
        .plot(
            BarPlot("Connection Loss Distance")
            .x(elapsed)
            .y(loss_distance)
            .with_unit("m")
            .with_group("Stability")
        )
        .plot(
            BarPlot("Median Distance By Serial")
            .x(dimension("serial_number"))
            .y(distance.median())
            .with_unit("m")
            .with_group("Cycle")
        )
        .plot(
            BarPlot("Samples By Serial")
            .x(dimension("serial_number"))
            .y(distance.count())
            .series_by(dimension("serial_number"))
        )
        .plot(
            BoxSummaryDeclaration("Distance Summary")
            .y(distance)
            .with_unit("m")
            .with_group("Statistics")
        )
    )


def main() -> None:
    document = build_plot_catalog_document().to_contract()
    document_cards = document["cards"]
    metadata = document["metadata"]
    assert isinstance(document_cards, list)
    assert isinstance(metadata, dict)

    plots = [card["data"]["plot"] for card in document_cards if card["type"] == "plot"]
    print(
        json.dumps(
            {
                "document": metadata["identifier"],
                "plot_types": [plot["plot_type"] for plot in plots],
                "plot_count": len(plots),
            },
            indent=2,
            sort_keys=True,
        )
    )


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
