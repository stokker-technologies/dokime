/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Unary Plot Analyses

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;
use sha2::{Digest, Sha256};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn query(context: &Context, request: v1::PlotRequest) -> Result<dokime_protocol::client::plots::PlotEnvelope> {
    if !matches!(v1::PlotKind::try_from(request.plot_type), Ok(v1::PlotKind::Unspecified | v1::PlotKind::TimeSeries))
        || request.x_key.as_ref().is_some_and(|key| key != "timestamp")
        || request.max_points == Some(0)
    {
        return Err(Error::Invalid("Query Analysis: Select a Time-Series Source and Positive Display Resolution".into()));
    }
    let mut selection = request.analysis.clone().ok_or_else(|| Error::Invalid("Query Analysis: Select Its ID and Version".into()))?;
    if request.y_keys.is_empty()
        || request.series_by.is_some()
        || request.series_by_expression.is_some()
        || !request.y_expressions.is_empty()
        || request.x_expression.is_some()
        || request.z_expression.is_some()
        || request.z_field.is_some()
        || request.z_stream.is_some()
        || !request.filters.is_empty()
    {
        return Err(Error::Invalid("Query Analysis: Select Raw Numeric Fields Without Grouping or Derived Expressions".into()));
    }
    let mut providers = Vec::new();
    for plugin in context.host.plugins()? {
        if plugin.state != dokime_protocol::client::PluginState::Ready as i32 {
            continue;
        }
        let Some(description) = plugin.description else {
            continue;
        };
        for section in description.catalog.into_iter().flat_map(|catalog| catalog.sections) {
            if section.r#type != "dokime.v1.Analyses" {
                continue;
            }
            for definition in v1::Analyses::decode(section.payload.as_slice())?.analyses {
                if definition.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.id)
                    && definition.version == selection.version
                {
                    providers.push((description.id.clone(), definition));
                }
            }
        }
    }
    if providers.len() != 1 {
        return Err(Error::Invalid("Query Analysis: Connect One Ready Plugin Declaring This Analysis ID and Version".into()));
    }
    let (provider, definition) = providers.remove(0);
    if definition.max_samples == 0 || definition.max_output_values == 0 || definition.outputs.is_empty() {
        return Err(Error::Invalid(
            "Query Analysis: Rebuild Its Catalog with Positive Input and Output Bounds and Declared Outputs".into(),
        ));
    }
    let parameters = selection.parameters.get_or_insert_default();
    for key in parameters.values.keys() {
        if !definition.parameters.iter().any(|parameter| parameter.metadata.as_ref().is_some_and(|metadata| metadata.identifier == *key)) {
            return Err(Error::Invalid(format!("Query Analysis: Remove Undeclared Parameter {key}")));
        }
    }
    for parameter in &definition.parameters {
        let key = parameter.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
        if !parameters.values.contains_key(key)
            && let Some(v1::parameter_expression::Kind::Literal(value)) =
                parameter.expression.as_ref().and_then(|expression| expression.kind.as_ref())
        {
            parameters.values.insert(key.into(), value.clone());
        }
        if !parameters.values.contains_key(key) && parameter.expression.is_some() {
            return Err(Error::Invalid(format!("Query Analysis: Supply a Resolved Value for Derived Default {key}")));
        }
        let Some(value) = parameters.values.get(key) else {
            if parameter.required {
                return Err(Error::Invalid(format!("Query Analysis: Supply Required Parameter {key}")));
            }
            continue;
        };
        let kind = match value.kind {
            Some(v1::value::Kind::Boolean(_)) => v1::ValueType::Boolean,
            Some(v1::value::Kind::Integer(_)) => v1::ValueType::Integer,
            Some(v1::value::Kind::UnsignedInteger(_)) => v1::ValueType::UnsignedInteger,
            Some(v1::value::Kind::Number(_)) => v1::ValueType::Number,
            Some(v1::value::Kind::Text(_)) => v1::ValueType::Text,
            _ => return Err(Error::Invalid(format!("Query Analysis: Supply a Scalar Value for {key}"))),
        };
        let numeric = match value.kind {
            Some(v1::value::Kind::Number(value)) => Some(value),
            Some(v1::value::Kind::Integer(value)) => Some(value as f64),
            Some(v1::value::Kind::UnsignedInteger(value)) => Some(value as f64),
            _ => None,
        };
        if kind as i32 != parameter.r#type
            || !parameter.choices.is_empty() && !parameter.choices.contains(value)
            || numeric.is_some_and(|value| {
                !value.is_finite() || parameter.min.is_some_and(|min| value < min) || parameter.max.is_some_and(|max| value > max)
            })
        {
            return Err(Error::Invalid(format!("Query Analysis: Supply {key} Within Its Declared Type, Choices and Bounds")));
        }
    }
    let input_limit = (definition.max_samples as usize).min(context.settings.page_items).min(context.settings.message_bytes / 16);
    if input_limit == 0 {
        return Err(Error::Invalid("Query Analysis: Increase the Configured Input Budget".into()));
    }
    let mut raw = request.clone();
    raw.analysis = None;
    raw.technique = v1::Downsample::Raw as i32;
    raw.max_points = Some(input_limit as u64);
    let mut envelope = super::plots_snapshots_list::query_base(context, raw).await?;
    let payload = envelope.payload.as_mut().ok_or_else(|| Error::Invalid("Query Analysis: Capture Its Raw Source Window".into()))?;
    let frame = payload.frame.take().ok_or_else(|| Error::Invalid("Query Analysis: Select a Time-Series Source".into()))?;
    if frame.series.len() != payload.series.len() {
        return Err(Error::Invalid("Query Analysis: Select One Scalar Column per Series".into()));
    }
    if frame.series.iter().map(|column| column.y.len()).sum::<usize>() > input_limit {
        return Err(Error::Invalid("Query Analysis: Narrow the Window or Select Fewer Fields Before Invoking the Plugin".into()));
    }
    let mut computed = Vec::new();
    let mut input_count = 0;
    let mut output_count = 0;
    let mut display_count = 0;
    for (column, series) in frame.series.into_iter().zip(&payload.series) {
        let values = column
            .y
            .into_iter()
            .map(|value| value.value.ok_or_else(|| Error::Invalid("Query Analysis: Select Samples Without Missing Values".into())))
            .collect::<Result<Vec<_>>>()?;
        let mut axis = column.x.iter().map(number).collect::<Result<Vec<_>>>()?;
        let origin = axis.first().copied().ok_or_else(|| Error::Invalid("Query Analysis: Capture Nonempty Raw Samples".into()))?;
        for value in &mut axis {
            *value -= origin;
        }
        if axis.windows(2).any(|pair| pair[0] >= pair[1]) || axis.iter().chain(&values).any(|value| !value.is_finite()) {
            return Err(Error::Invalid("Query Analysis: Select Finite Samples with a Strictly Increasing Source-Time Axis".into()));
        }
        input_count += values.len();
        if input_count > input_limit {
            return Err(Error::Invalid(
                "Query Analysis: Narrow the Window or Select Fewer Columns to Fit the Aggregate Input Bound".into(),
            ));
        }
        let call = v1::AnalysisRequest {
            samples: Some(v1::AnalysisSamples {
                axis: axis.clone(),
                values,
                axis_unit: "s".into(),
                unit: series.unit.clone().unwrap_or_default(),
            }),
            parameters: selection.parameters.clone(),
        };
        if call.encoded_len() > context.settings.message_bytes {
            return Err(Error::Invalid("Query Analysis: Narrow Its Input to Fit the Message Budget".into()));
        }
        let result = context.host.analyze(&provider, &selection.id, call).await?;
        if result.outputs.len() != definition.outputs.len() {
            return Err(Error::Invalid("Query Analysis: Return Every Declared Output Exactly Once".into()));
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut outputs = Vec::new();
        for mut output in result.outputs {
            let declaration = definition
                .outputs
                .iter()
                .find(|declaration| declaration.metadata.as_ref().is_some_and(|metadata| metadata.identifier == output.key))
                .ok_or_else(|| Error::Invalid("Query Analysis: Remove Undeclared Outputs from the Plugin Reply".into()))?;
            if output.values.is_empty()
                || !seen.insert(output.key.clone())
                || !declaration.unit.is_empty() && declaration.unit != output.unit
                || !declaration.axis_unit.is_empty() && declaration.axis_unit != output.axis_unit
                || output.values.iter().chain(&output.axis).any(|value| !value.is_finite())
                || if declaration.axis.is_empty() {
                    !output.axis.is_empty() || output.values.len() != 1
                } else {
                    output.axis.len() != output.values.len() || output.axis.windows(2).any(|pair| pair[0] >= pair[1])
                }
            {
                return Err(Error::Invalid("Query Analysis: Return Unique Finite Outputs Matching Their Declared Axes and Units".into()));
            }
            output_count += output.values.len();
            if output_count > definition.max_output_values as usize || output_count > context.settings.message_bytes / 8 {
                return Err(Error::Invalid("Query Analysis: Reduce Plugin Output to Its Declared Aggregate Bound".into()));
            }
            let display =
                request.max_points.unwrap_or(context.settings.page_items as u64).max(2).min(context.settings.page_items as u64) as usize;
            let length = output.values.len();
            output.display_indices = (0..length.min(display))
                .map(|index| if length <= display { index as u32 } else { (index * (length - 1) / (display - 1)) as u32 })
                .collect();
            display_count += output.display_indices.len();
            outputs.push(output);
        }
        computed.push(dokime_protocol::client::plots::AnalysisSeries {
            key: series.key.clone(),
            label: series.label.clone(),
            origin: Some(super::projection::timestamp((origin * 1e9) as i64).to_string()),
            input_samples: axis.len() as u64,
            first_axis: axis[0],
            last_axis: *axis.last().unwrap(),
            outputs,
        });
    }
    payload.computed = Some(dokime_protocol::client::plots::AnalysisComputation {
        definition: Some(definition),
        selection: Some(selection),
        source: Some(dokime_protocol::client::plots::AnalysisEvidence {
            store: context.host.identity().backend_id,
            epoch: super::projection::execution_scope(request.scope.as_ref())?.into(),
            position: envelope.revision,
            stream: request.stream.clone(),
            scope: request.scope.clone(),
            window_s: request.window_s,
            fields: request.y_keys.clone(),
            x_key: payload.x_key.clone(),
        }),
        series: computed,
    });
    payload.bucket_width_s = None;
    payload.max_points = request.max_points.unwrap_or(context.settings.page_items as u64);
    envelope.selection = Sha256::digest(request.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect();
    if let Some(diagnostics) = &mut envelope.diagnostics {
        diagnostics.point_count = display_count as u64;
        diagnostics.max_points = payload.max_points;
        diagnostics.resolution_nanos = 0;
        diagnostics.downsampled = input_count > display_count;
    }
    if envelope.encoded_len() > context.settings.message_bytes {
        return Err(Error::Invalid("Query Analysis: Reduce Its Declared Output or Split the Series to Fit One Message".into()));
    }
    Ok(envelope)
}

/* === Private Functions === */

fn number(value: &v1::Value) -> Result<f64> {
    match value.kind {
        Some(v1::value::Kind::Number(value)) => Ok(value),
        _ => Err(Error::Invalid("Query Analysis: Select a Numeric Source-Time Axis".into())),
    }
}

/* === Tests === */
