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
use tonic::Status;
// Local Imports
use dokime_app::{
    ExecutionFileContentKind, ExecutionFileStreamState,
    core::ValueType,
    document::{
        AxisScale, FftDetrend, FftWindow, FilterOperator, FrequencyLevelUnit, PlotAxisKind,
        PlotStyle, PlotType, ReferenceAxis,
    },
    runtime::query::DownsampleTechnique,
};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */
macro_rules! encode {
    ($name:ident: $native:ident -> $wire:ident { $($variant:ident),+ $(,)? }) => {
        pub(super) fn $name(value: &$native) -> v1::$wire {
            match value {
                $($native::$variant => v1::$wire::$variant,)+
            }
        }
    };
}

macro_rules! convert {
    (
        $encode:ident / $decode:ident: $native:ident <-> $wire:ident as $noun:literal
        { $($variant:ident),+ $(,)? }
    ) => {
        encode!($encode: $native -> $wire { $($variant),+ });

        pub(super) fn $decode(value: i32) -> Result<Option<$native>, Status> {
            match v1::$wire::try_from(value)
                .map_err(|_| Status::invalid_argument(concat!("unknown ", $noun)))?
            {
                v1::$wire::Unspecified => Ok(None),
                $(v1::$wire::$variant => Ok(Some($native::$variant)),)+
            }
        }
    };
}

convert!(plot_type / plot_type_from_wire: PlotType <-> PlotType as "plot type" {
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

convert!(axis_scale / axis_scale_from_wire: AxisScale <-> AxisScale as "axis scale" {
    Linear,
    Log,
});

convert!(
    filter_operator / filter_operator_from_wire: FilterOperator <-> FilterOperator
        as "plot filter operator"
    {
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

convert!(
    downsample_technique / downsample_technique_from_wire: DownsampleTechnique
        <-> DownsampleTechnique as "downsample technique"
    {
        Raw,
        MinMax,
        Lttb,
    }
);

encode!(plot_style: PlotStyle -> PlotStyle { Line, Points, LinePoints, Step });

encode!(fft_window: FftWindow -> FftWindow { Hann, Rectangular });

encode!(fft_detrend: FftDetrend -> FftDetrend { None, Mean });

encode!(frequency_level_unit: FrequencyLevelUnit -> FrequencyLevelUnit {
    Linear,
    Db,
    Dbu,
    Dbm,
});

encode!(reference_axis: ReferenceAxis -> ReferenceAxis { X, Y });

pub(super) fn plot_axis(value: Option<&PlotAxisKind>) -> (i32, Option<String>) {
    match value {
        None => (v1::PlotAxisKind::Unspecified as i32, None),
        Some(PlotAxisKind::WallTime) => (v1::PlotAxisKind::WallTime as i32, None),
        Some(PlotAxisKind::Elapsed { unit }) => {
            (v1::PlotAxisKind::Elapsed as i32, Some(unit.clone()))
        }
    }
}

encode!(execution_file_kind: ExecutionFileContentKind -> ExecutionFileKind {
    Text,
    Binary,
    Media,
    Unknown,
});

encode!(execution_file_state: ExecutionFileStreamState -> ExecutionFileState {
    Streaming,
    Finalized,
    Interrupted,
});

encode!(value_type: ValueType -> ValueType {
    Boolean,
    Integer,
    Number,
    Text,
    Datetime,
    File,
    Object,
    Array,
});

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
