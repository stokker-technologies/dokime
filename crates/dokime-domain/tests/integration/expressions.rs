/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A data expression's aggregate is the Harmos vocabulary under its wire name,
// the same in JSON, in the expression tree and back.

use harmos_domain::Aggregate;

use dokime_domain::model;
use model::expressions::{DataExpression, DataTransform, Expression, telemetry};

const VOCABULARY: [Aggregate; 11] = [
    Aggregate::Count,
    Aggregate::Sum,
    Aggregate::Mean,
    Aggregate::Median,
    Aggregate::Min,
    Aggregate::Max,
    Aggregate::Rms,
    Aggregate::Std,
    Aggregate::Percentile(0),
    Aggregate::Percentile(95),
    Aggregate::Percentile(100),
];

// Every member serializes tagged `aggregate` under its name and reads back.
#[test]
fn aggregate_json_roundtrips() {
    for aggregate in VOCABULARY {
        let transform = DataTransform::Aggregate(aggregate);
        let json = serde_json::to_value(&transform).unwrap();
        assert_eq!(json, serde_json::json!({ "type": "aggregate", "aggregate": aggregate.name() }));
        assert_eq!(serde_json::from_value::<DataTransform>(json).unwrap(), transform);
    }

    let unknown = serde_json::json!({ "type": "aggregate", "aggregate": "p101" });
    assert!(serde_json::from_value::<DataTransform>(unknown).is_err());
}

// The builder methods name the vocabulary and survive the expression tree.
#[test]
fn builders_roundtrip_expressions() {
    let built = [
        telemetry("bench", "v").count(),
        telemetry("bench", "v").sum(),
        telemetry("bench", "v").mean(),
        telemetry("bench", "v").median(),
        telemetry("bench", "v").min(),
        telemetry("bench", "v").max(),
        telemetry("bench", "v").rms(),
        telemetry("bench", "v").std(),
        telemetry("bench", "v").percentile(95).per_case(),
    ];
    for expression in built {
        let tree = Expression::from(&expression);
        assert_eq!(DataExpression::try_from(tree).unwrap(), expression);
    }

    assert_eq!(telemetry("bench", "v").percentile(95).transforms, [DataTransform::Aggregate(Aggregate::Percentile(95))]);
}
