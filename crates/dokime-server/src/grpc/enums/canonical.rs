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
use dokime_domain::operations::plots;
use dokime_runtime::{
    ExecutionFileContentKind, ExecutionFileStreamState,
    document::{AxisScale, FftDetrend, FftWindow, FilterOperator, FrequencyLevelUnit, PlotAxisKind, PlotStyle, PlotType, ReferenceAxis},
    runtime::query::DownsampleTechnique,
};

/* === Constants === */

/* === Macros === */

macro_rules! encode {
    ($name:ident: $native:ident -> $wire:ty { $($variant:ident),+ $(,)? }) => {
        pub(in crate::grpc) fn $name(value: &$native) -> $wire {
            match value { $($native::$variant => <$wire>::$variant,)+ }
        }
    };
}

encode!(plot_type: PlotType -> dokime_domain::model::plots::wire::PlotType {
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

encode!(axis_scale: AxisScale -> dokime_domain::model::plots::wire::AxisScale {
    Linear,
    Log,
});

encode!(
    filter_operator: FilterOperator -> dokime_domain::model::plots::wire::FilterOperator {
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

encode!(plot_style: PlotStyle -> dokime_domain::model::documents::wire::PlotStyle { Line, Points, LinePoints, Step });

encode!(fft_window: FftWindow -> dokime_domain::model::plots::wire::FftWindow { Hann, Rectangular });

encode!(fft_detrend: FftDetrend -> dokime_domain::model::plots::wire::FftDetrend { None, Mean });

encode!(frequency_level_unit: FrequencyLevelUnit -> dokime_domain::model::plots::wire::FrequencyLevelUnit {
    Linear,
    Db,
    Dbu,
    Dbm,
});

encode!(reference_axis: ReferenceAxis -> dokime_domain::model::documents::wire::ReferenceAxis { X, Y });

encode!(execution_file_kind: ExecutionFileContentKind -> dokime_domain::model::content::wire::ExecutionFileKind {
    Text,
    Binary,
    Media,
    Unknown,
});

encode!(execution_file_state: ExecutionFileStreamState -> dokime_domain::model::content::wire::ExecutionFileState {
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

pub(in crate::grpc) fn plot_axis(value: Option<&PlotAxisKind>) -> (dokime_domain::model::documents::wire::PlotAxisKind, Option<String>) {
    match value {
        None => (dokime_domain::model::documents::wire::PlotAxisKind::Unspecified, None),
        Some(PlotAxisKind::WallTime) => (dokime_domain::model::documents::wire::PlotAxisKind::WallTime, None),
        Some(PlotAxisKind::Elapsed { unit }) => (dokime_domain::model::documents::wire::PlotAxisKind::Elapsed, Some(unit.clone())),
    }
}

pub(in crate::grpc) fn plot_type_from_wire(value: dokime_domain::model::plots::wire::PlotType) -> Result<Option<PlotType>, tonic::Status> {
    Ok(match value {
        dokime_domain::model::plots::wire::PlotType::Unspecified => None,
        dokime_domain::model::plots::wire::PlotType::TimeSeries => Some(PlotType::TimeSeries),
        dokime_domain::model::plots::wire::PlotType::Bar => Some(PlotType::Bar),
        dokime_domain::model::plots::wire::PlotType::GroupedBar => Some(PlotType::GroupedBar),
        dokime_domain::model::plots::wire::PlotType::StackedBar => Some(PlotType::StackedBar),
        dokime_domain::model::plots::wire::PlotType::Pie => Some(PlotType::Pie),
        dokime_domain::model::plots::wire::PlotType::Donut => Some(PlotType::Donut),
        dokime_domain::model::plots::wire::PlotType::Pareto => Some(PlotType::Pareto),
        dokime_domain::model::plots::wire::PlotType::XyScatter => Some(PlotType::XyScatter),
        dokime_domain::model::plots::wire::PlotType::XyLine => Some(PlotType::XyLine),
        dokime_domain::model::plots::wire::PlotType::XyInterval => Some(PlotType::XyInterval),
        dokime_domain::model::plots::wire::PlotType::Trajectory => Some(PlotType::Trajectory),
        dokime_domain::model::plots::wire::PlotType::Heatmap => Some(PlotType::Heatmap),
        dokime_domain::model::plots::wire::PlotType::XyValueHeatmap => Some(PlotType::XyValueHeatmap),
        dokime_domain::model::plots::wire::PlotType::WifiChannelOccupancy => Some(PlotType::WifiChannelOccupancy),
        dokime_domain::model::plots::wire::PlotType::FrequencySeries => Some(PlotType::FrequencySeries),
        dokime_domain::model::plots::wire::PlotType::Histogram => Some(PlotType::Histogram),
        dokime_domain::model::plots::wire::PlotType::BoxSummary => Some(PlotType::BoxSummary),
        dokime_domain::model::plots::wire::PlotType::AggregateTable => Some(PlotType::AggregateTable),
    })
}

pub(in crate::grpc) fn axis_scale_from_wire(
    value: dokime_domain::model::plots::wire::AxisScale,
) -> Result<Option<AxisScale>, tonic::Status> {
    Ok(match value {
        dokime_domain::model::plots::wire::AxisScale::Unspecified => None,
        dokime_domain::model::plots::wire::AxisScale::Linear => Some(AxisScale::Linear),
        dokime_domain::model::plots::wire::AxisScale::Log => Some(AxisScale::Log),
    })
}

pub(in crate::grpc) fn filter_operator_from_wire(
    value: dokime_domain::model::plots::wire::FilterOperator,
) -> Result<Option<FilterOperator>, tonic::Status> {
    Ok(match value {
        dokime_domain::model::plots::wire::FilterOperator::Unspecified => None,
        dokime_domain::model::plots::wire::FilterOperator::Eq => Some(FilterOperator::Eq),
        dokime_domain::model::plots::wire::FilterOperator::Ne => Some(FilterOperator::Ne),
        dokime_domain::model::plots::wire::FilterOperator::Gt => Some(FilterOperator::Gt),
        dokime_domain::model::plots::wire::FilterOperator::Gte => Some(FilterOperator::Gte),
        dokime_domain::model::plots::wire::FilterOperator::Lt => Some(FilterOperator::Lt),
        dokime_domain::model::plots::wire::FilterOperator::Lte => Some(FilterOperator::Lte),
        dokime_domain::model::plots::wire::FilterOperator::Contains => Some(FilterOperator::Contains),
        dokime_domain::model::plots::wire::FilterOperator::Exists => Some(FilterOperator::Exists),
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
