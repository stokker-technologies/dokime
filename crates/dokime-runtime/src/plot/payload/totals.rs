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
use std::collections::BTreeSet;

// Third Party Imports
use serde_json::Value;

// Local Imports
use super::{PlotPayloadRequest, PlotSeries};
use crate::document::{PlotType, validate_plot_options};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn validate(series: &[PlotSeries], request: &PlotPayloadRequest) -> Result<(), String> {
    let Some(kind) = &request.plot_type else { return Ok(()) };
    validate_plot_options(
        kind,
        request.arrangement,
        request.pareto,
        request.hole,
        request.xy_interval,
        request.series_by.is_some() || request.series_by_expression.is_some(),
    )
    .map_err(str::to_owned)?;
    if *kind != PlotType::Pie && !request.pareto {
        return Ok(());
    }
    if request.y_keys.len() + request.y_expressions.len() != 1 {
        return Err("PIE and Pareto require one numeric total per category".into());
    }
    let mut categories = BTreeSet::new();
    for point in series.iter().flat_map(|series| &series.points) {
        let category = match &point.x {
            Value::String(value) => value.clone(),
            Value::Number(value) => value.as_f64().map(|value| value.to_string()).ok_or("category must be finite")?,
            Value::Bool(value) => value.to_string(),
            _ => return Err("PIE and Pareto require a category for every total".into()),
        };
        if !categories.insert(category.clone()) {
            return Err(format!("duplicate category total {category}; aggregate samples explicitly"));
        }
        if !point.y.is_some_and(|value| value.is_finite() && value >= 0.0) {
            return Err(format!("category {category} requires a finite nonnegative total"));
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::super::PlotPoint;
    use super::*;
    use crate::document::BarArrangement;

    #[test]
    fn category_totals_refuse_duplicates_invalid_values_and_conflicting_options() {
        let mut request = PlotPayloadRequest {
            plot_type: Some(PlotType::Pie),
            y_keys: vec!["total".into()],
            ..Default::default()
        };
        let mut series = vec![PlotSeries {
            key: "total".into(),
            label: "Total".into(),
            unit: None,
            points: vec![],
            raw_points: 0,
            downsampled: false,
            format: None,
        }];
        for value in [0.0, 2.0] {
            series[0].points = vec![PlotPoint {
                x: "a".into(),
                y: Some(value),
                z: None,
                instant: None,
            }];
            validate(&series, &request).unwrap();
        }
        for value in [-1.0, f64::NAN, f64::INFINITY] {
            series[0].points[0].y = Some(value);
            assert!(validate(&series, &request).is_err());
        }
        series[0].points[0].y = Some(2.0);
        let duplicate = series[0].points[0].clone();
        series[0].points.push(duplicate);
        assert!(validate(&series, &request).is_err());
        series[0].points.pop();
        for hole in [-0.1, 1.0, f64::NAN, f64::INFINITY] {
            request.hole = Some(hole);
            assert!(validate(&series, &request).is_err());
        }
        request.hole = None;
        request.plot_type = Some(PlotType::Bar);
        request.pareto = true;
        request.arrangement = Some(BarArrangement::Grouped);
        assert!(validate(&series, &request).is_err());
        request.arrangement = None;
        request.series_by = Some("group".into());
        assert!(validate(&series, &request).is_err());
        request.series_by = None;
        validate(&series, &request).unwrap();
    }
}
