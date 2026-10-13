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

// Third Party Imports

// Local Imports
use dokime_app::{
    ExecutionFileContentKind, ExecutionFileStreamState,
    document::{
        AxisScale, FftDetrend, FftWindow, FilterOperator, FrequencyLevelUnit, PlotAxisKind,
        PlotStyle, PlotType, ReferenceAxis,
    },
    runtime::query::DownsampleTechnique,
};
use dokime_contract::operations::{content, documents, plots};

/* === Constants === */

/* === Macros === */
macro_rules! encode {
    ($name:ident: $native:ident -> $owner:ident::$wire:ident { $($variant:ident),+ $(,)? }) => {
        pub(in crate::grpc) fn $name(value: &$native) -> $owner::$wire {
            match value { $($native::$variant => $owner::$wire::$variant,)+ }
        }
    };
}

encode!(plot_type: PlotType -> plots::PlotType {
    TimeSeries,
    Bar,
    GroupedBar,
    StackedBar,
    Pie,
    Donut,
    Pareto,
    XyScatter,
    XyLine,
    XyInterval,
    Trajectory,
    Heatmap,
    XyValueHeatmap,
    WifiChannelOccupancy,
    FrequencySeries,
    Histogram,
    BoxSummary,
    AggregateTable,
});

encode!(axis_scale: AxisScale -> plots::AxisScale {
    Linear,
    Log,
});

encode!(
    filter_operator: FilterOperator -> plots::FilterOperator {
        Eq,
        Ne,
        Gt,
        Gte,
        Lt,
        Lte,
        Contains,
        Exists,
    }
);

encode!(
    downsample_technique: DownsampleTechnique -> plots::DownsampleTechnique {
        Unspecified,
        Auto,
        Raw,
        MinMax,
        Lttb,
    }
);

encode!(plot_style: PlotStyle -> documents::PlotStyle { Line, Points, LinePoints, Step });

encode!(fft_window: FftWindow -> plots::FftWindow { Hann, Rectangular });

encode!(fft_detrend: FftDetrend -> plots::FftDetrend { None, Mean });

encode!(frequency_level_unit: FrequencyLevelUnit -> plots::FrequencyLevelUnit {
    Linear,
    Db,
    Dbu,
    Dbm,
});

encode!(reference_axis: ReferenceAxis -> documents::ReferenceAxis { X, Y });

pub(in crate::grpc) fn plot_axis(
    value: Option<&PlotAxisKind>,
) -> (documents::PlotAxisKind, Option<String>) {
    match value {
        None => (documents::PlotAxisKind::Unspecified, None),
        Some(PlotAxisKind::WallTime) => (documents::PlotAxisKind::WallTime, None),
        Some(PlotAxisKind::Elapsed { unit }) => {
            (documents::PlotAxisKind::Elapsed, Some(unit.clone()))
        }
    }
}

encode!(execution_file_kind: ExecutionFileContentKind -> content::ExecutionFileKind {
    Text,
    Binary,
    Media,
    Unknown,
});

encode!(execution_file_state: ExecutionFileStreamState -> content::ExecutionFileState {
    Streaming,
    Finalized,
    Interrupted,
});

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::grpc) fn plot_type_from_wire(
    value: plots::PlotType,
) -> Result<Option<PlotType>, tonic::Status> {
    Ok(match value {
        plots::PlotType::Unspecified => None,
        plots::PlotType::TimeSeries => Some(PlotType::TimeSeries),
        plots::PlotType::Bar => Some(PlotType::Bar),
        plots::PlotType::GroupedBar => Some(PlotType::GroupedBar),
        plots::PlotType::StackedBar => Some(PlotType::StackedBar),
        plots::PlotType::Pie => Some(PlotType::Pie),
        plots::PlotType::Donut => Some(PlotType::Donut),
        plots::PlotType::Pareto => Some(PlotType::Pareto),
        plots::PlotType::XyScatter => Some(PlotType::XyScatter),
        plots::PlotType::XyLine => Some(PlotType::XyLine),
        plots::PlotType::XyInterval => Some(PlotType::XyInterval),
        plots::PlotType::Trajectory => Some(PlotType::Trajectory),
        plots::PlotType::Heatmap => Some(PlotType::Heatmap),
        plots::PlotType::XyValueHeatmap => Some(PlotType::XyValueHeatmap),
        plots::PlotType::WifiChannelOccupancy => Some(PlotType::WifiChannelOccupancy),
        plots::PlotType::FrequencySeries => Some(PlotType::FrequencySeries),
        plots::PlotType::Histogram => Some(PlotType::Histogram),
        plots::PlotType::BoxSummary => Some(PlotType::BoxSummary),
        plots::PlotType::AggregateTable => Some(PlotType::AggregateTable),
    })
}
pub(in crate::grpc) fn axis_scale_from_wire(
    value: plots::AxisScale,
) -> Result<Option<AxisScale>, tonic::Status> {
    Ok(match value {
        plots::AxisScale::Unspecified => None,
        plots::AxisScale::Linear => Some(AxisScale::Linear),
        plots::AxisScale::Log => Some(AxisScale::Log),
    })
}
pub(in crate::grpc) fn filter_operator_from_wire(
    value: plots::FilterOperator,
) -> Result<Option<FilterOperator>, tonic::Status> {
    Ok(match value {
        plots::FilterOperator::Unspecified => None,
        plots::FilterOperator::Eq => Some(FilterOperator::Eq),
        plots::FilterOperator::Ne => Some(FilterOperator::Ne),
        plots::FilterOperator::Gt => Some(FilterOperator::Gt),
        plots::FilterOperator::Gte => Some(FilterOperator::Gte),
        plots::FilterOperator::Lt => Some(FilterOperator::Lt),
        plots::FilterOperator::Lte => Some(FilterOperator::Lte),
        plots::FilterOperator::Contains => Some(FilterOperator::Contains),
        plots::FilterOperator::Exists => Some(FilterOperator::Exists),
    })
}
pub(in crate::grpc) fn downsample_technique_from_wire(
    value: plots::DownsampleTechnique,
) -> Result<Option<DownsampleTechnique>, tonic::Status> {
    Ok(match value {
        plots::DownsampleTechnique::Unspecified => Some(DownsampleTechnique::Auto),
        plots::DownsampleTechnique::Auto => Some(DownsampleTechnique::Auto),
        plots::DownsampleTechnique::Raw => Some(DownsampleTechnique::Raw),
        plots::DownsampleTechnique::MinMax => Some(DownsampleTechnique::MinMax),
        plots::DownsampleTechnique::Lttb => Some(DownsampleTechnique::Lttb),
    })
}
/* === Private Functions === */

/* === Tests === */
