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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use dokime_runtime::definition::{
    CaseMaterializationOptions, CaseMaterializationSelection, ParameterDefinition, ParameterExpr, ParameterList, ParameterResolverError,
    ParameterSelection, ParameterSource, TemplateContext, authored_metadata, materialize_template_cases, materialized_parameter,
    resolve_parameter_spaces,
};
use dokime_runtime::expressions::DataExpression;
use dokime_runtime::{FromParameters, Parameter, ParameterDefault, Value, ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::parameters]
pub struct HoldTimes {
    #[parameter(
        label = "Hold times",
        default = vec![30, 60, 120],
        min_items = 1,
        max_items = 8,
    )]
    pub hold_times: Vec<i64>,
}

#[dokime_runtime::parameters]
pub struct Soak {
    #[parameter(label = "Target temperature", unit = "°C", default = 60.0, min = -50.0, max = 200.0)]
    pub target_c: f64,
    #[parameter(default = 5, min = 1)]
    pub hold_samples: u32,
    #[parameter(label = "Operator")]
    pub operator: Option<String>,
    #[parameter(min_items = 1, max_items = 2)]
    pub ports: Vec<String>,
    #[parameter(label = "How the sweep runs")]
    pub sweep: Sweep,
}

#[dokime_runtime::parameters]
pub struct Sweeps {
    #[parameter(label = "Sweeps to run", min_items = 1)]
    pub sweeps: Vec<Sweep>,
    #[parameter(label = "Fallback sweep")]
    pub fallback: Option<Sweep>,
}

#[dokime_runtime::record(id = "parameters.sweep_selected", label = "Sweep Selected")]
struct SweepSelected {
    sweep: Sweep,
}

#[dokime_runtime::stream(id = "parameters.sweep_samples", window = 4096, label = "Sweep Samples")]
#[derive(Debug)]
struct SweepSample {
    sweep: Sweep,
}

#[dokime_runtime::parameters]
struct ExpansionInputs {
    #[parameter(expansion = "cartesian", default = true)]
    fixed: bool,
    #[parameter(default = vec!["A", "B"])]
    names: Vec<String>,
    #[parameter(expansion = "cartesian", default = [false, true])]
    enabled: bool,
    #[parameter(expansion = zip("point"), default = [1, 2])]
    x: i64,
    #[parameter(expansion = zip("point"), default = [10, 20])]
    y: i64,
    #[parameter(expansion = "cartesian", default = vec![vec!["A", "B"], vec!["C"]])]
    batches: Vec<String>,
}

/* === Enums === */
#[dokime_runtime::choice]
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Sweep {
    Fast,
    #[choice(label = "Slow and careful")]
    Slow,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn template(parameters: Vec<ParameterDefinition>) -> dokime_runtime::definition::TestCaseTemplate {
    let mut ctx = TemplateContext::new();
    ctx.set_metadata(authored_metadata("parameters.example", "Example", None));
    ctx.set_parameter_definitions(parameters);
    ctx.into_test_case_template().unwrap()
}

fn case() -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("target_c".to_owned(), Value::Number(60.0)),
        ("hold_samples".to_owned(), Value::Integer(5)),
        ("operator".to_owned(), Value::Null),
        ("ports".to_owned(), Value::Array(vec![Value::from("/dev/ttyUSB0")])),
        ("sweep".to_owned(), Value::from("fast")),
    ])
}

/* === Tests === */

#[test]
fn optional_values() {
    assert_eq!(Value::from(None::<f64>), Value::Null);
    assert_eq!(Value::from(Some(12.5)), Value::Number(12.5));
    assert_eq!(Value::from(Some("")), Value::Text(String::new()));
    assert_eq!(Value::from(Some(false)), Value::Bool(false));
    assert_eq!(Value::from(Some(0)), Value::Integer(0));
    assert_eq!(serde_json::to_string(&Value::from(None::<String>)).unwrap(), r#"{"type":"null"}"#);

    for kind in
        [ValueType::Boolean, ValueType::Integer, ValueType::Number, ValueType::Text, ValueType::File, ValueType::Array, ValueType::Object]
    {
        let mut parameter = Parameter::new("input", "Input", kind).optional().default(None::<String>).try_into_definition().unwrap();
        parameter.selection = Some(ParameterSelection { choices: vec![Value::from("choice")] });
        assert_eq!(resolve_parameter_spaces(&[parameter.clone()], &|_| None).unwrap()["input"].values, vec![Value::Null]);
        assert_eq!(materialized_parameter(&parameter, ParameterSource::Operator, Value::Null).unwrap().value, Value::Null);
        parameter.required = true;
        assert!(matches!(
            resolve_parameter_spaces(&[parameter.clone()], &|_| None),
            Err(dokime_runtime::ApiError::Parameter(ParameterResolverError::MissingValue { .. }))
        ));
        assert!(matches!(
            materialized_parameter(&parameter, ParameterSource::Operator, Value::Null),
            Err(dokime_runtime::ApiError::Parameter(ParameterResolverError::MissingValue { .. }))
        ));
    }
}

#[test]
fn overrides_and_sweeps() {
    let required = Parameter::number("voltage", "Voltage")
        .expansion(dokime_runtime::definition::ParameterExpansion::Cartesian)
        .try_into_definition()
        .unwrap();
    let dependent = Parameter::number("copy", "Copy")
        .expansion(dokime_runtime::definition::ParameterExpansion::Cartesian)
        .default(ParameterExpr::Reference { parameter: "voltage".into() })
        .try_into_definition()
        .unwrap();
    let optional = Parameter::text("serial", "Serial").optional().try_into_definition().unwrap();
    let template = template(vec![required, dependent, optional]);
    assert!(
        materialize_template_cases(
            "fixture.registration",
            &template,
            Default::default(),
            dokime_runtime::definition::CaseMaterializationOptions {
                max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
                force: false,
                registration_id: dokime_runtime::model::identity::new_id(),
                first_label_number: 1
            }
        )
        .is_err()
    );
    let selection = CaseMaterializationSelection {
        case_overrides: BTreeMap::from([("voltage".into(), vec![Value::from(12.0), Value::from(24.0)])]),
        ..Default::default()
    };
    let output = materialize_template_cases(
        "fixture.registration",
        &template,
        selection,
        CaseMaterializationOptions {
            max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
            force: false,
            registration_id: dokime_runtime::model::identity::new_id(),
            first_label_number: 1,
        },
    )
    .unwrap();
    assert_eq!(output.case_count, 4);
    let labels = output.test_cases.iter().map(|case| case.metadata.label.as_str()).collect::<Vec<_>>();
    let label = &template.metadata.label;
    assert_eq!(labels, (1..=4).map(|n| format!("{label} #{n}")).collect::<Vec<_>>());
    for case in &output.test_cases {
        assert_eq!(case.parameters["serial"], Value::Null);
        assert!([Value::from(12.0), Value::from(24.0)].contains(&case.parameters["copy"]));
    }
    for bad in [Value::Null, Value::from("invalid")] {
        let selection =
            CaseMaterializationSelection { case_overrides: BTreeMap::from([("voltage".into(), vec![bad])]), ..Default::default() };
        assert!(
            materialize_template_cases(
                "fixture.registration",
                &template,
                selection,
                dokime_runtime::definition::CaseMaterializationOptions {
                    max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
                    force: false,
                    registration_id: dokime_runtime::model::identity::new_id(),
                    first_label_number: 1
                }
            )
            .is_err()
        );
    }
    let sweep = Parameter::integer("count", "Count")
        .optional()
        .default(ParameterDefault::space([Some(0), None, Some(2)]))
        .try_into_definition()
        .unwrap();
    assert_eq!(
        resolve_parameter_spaces(&[sweep], &|_| None).unwrap()["count"].values,
        vec![Value::Integer(0), Value::Null, Value::Integer(2)]
    );
}

#[test]
fn optional_choices() {
    let mut parameter = Parameter::text("mode", "Mode").optional().default("fast").try_into_definition().unwrap();
    parameter.selection = Some(ParameterSelection { choices: vec![Value::from("fast")] });
    for multiple in [false, true] {
        parameter.list = multiple.then_some(dokime_runtime::definition::ParameterList {
            element_type: ValueType::Text,
            min_items: None,
            max_items: None,
        });
        parameter.value_type = if multiple { ValueType::Array } else { ValueType::Text };
        parameter.expression =
            ParameterExpr::Literal { value: if multiple { Value::Array(vec![Value::from("fast")]) } else { Value::from("fast") } };
        let template = template(vec![parameter.clone()]);
        let filter =
            CaseMaterializationSelection { case_filter: BTreeMap::from([("mode".into(), vec![Value::Null])]), ..Default::default() };
        assert!(
            materialize_template_cases(
                "fixture.registration",
                &template,
                filter,
                dokime_runtime::definition::CaseMaterializationOptions {
                    max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
                    force: false,
                    registration_id: dokime_runtime::model::identity::new_id(),
                    first_label_number: 1
                }
            )
            .is_err()
        );
        let selection =
            CaseMaterializationSelection { case_overrides: BTreeMap::from([("mode".into(), vec![Value::Null])]), ..Default::default() };
        assert_eq!(
            materialize_template_cases(
                "fixture.registration",
                &template,
                selection,
                dokime_runtime::definition::CaseMaterializationOptions {
                    max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
                    force: false,
                    registration_id: dokime_runtime::model::identity::new_id(),
                    first_label_number: 1
                }
            )
            .unwrap()
            .test_cases[0]
                .parameters["mode"],
            Value::Null
        );
    }
}

#[test]
fn list_param_vec() {
    let definition = Parameter::list("hold_times", "Hold times", ValueType::Integer)
        .min_items(1)
        .max_items(8)
        .default(vec![30, 60, 120])
        .try_into_definition()
        .unwrap();

    assert_eq!(definition.value_type, ValueType::Array);
    assert_eq!(definition.list, Some(ParameterList { element_type: ValueType::Integer, min_items: Some(1), max_items: Some(8) }));

    let template = template(vec![definition]);
    let output = materialize_template_cases(
        "fixture.registration",
        &template,
        Default::default(),
        dokime_runtime::definition::CaseMaterializationOptions {
            max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
            force: false,
            registration_id: dokime_runtime::model::identity::new_id(),
            first_label_number: 1,
        },
    )
    .unwrap();
    assert_eq!(output.case_count, 1);

    let parameters = output.test_cases[0].parameters.clone();
    assert_eq!(parameters["hold_times"], Value::Array(vec![Value::Integer(30), Value::Integer(60), Value::Integer(120),]));

    // The same declaration a `Vec<i64>` field writes, and the same value it
    // decodes into: the builder is the programmatic path to one declaration,
    // not a second one.
    assert_eq!(
        HoldTimes::declaration().into_iter().map(|parameter| parameter.try_into_definition().unwrap().list).collect::<Vec<_>>(),
        vec![Some(ParameterList { element_type: ValueType::Integer, min_items: Some(1), max_items: Some(8) })]
    );
    assert_eq!(HoldTimes::from_parameters(&parameters).unwrap().hold_times, vec![30, 60, 120]);
}

#[test]
fn list_refuses_value() {
    let definition = Parameter::list("hold_times", "Hold times", ValueType::Integer).try_into_definition().unwrap();

    let error = materialized_parameter(&definition, ParameterSource::Operator, Value::Integer(30)).unwrap_err();

    assert_eq!(error.to_string(), "list parameter \"hold_times\" expects a list of values, got Integer(30)");

    // The untyped array parameter this does not change still wraps a scalar.
    let untyped = Parameter::array("bands", "Bands").try_into_definition().unwrap();
    assert_eq!(
        materialized_parameter(&untyped, ParameterSource::Operator, Value::from("2.4")).unwrap().value,
        Value::Array(vec![Value::from("2.4")])
    );
}

#[test]
fn list_choices_optionality() {
    let definition =
        Parameter::list("bands", "Bands", ValueType::Text).choices(["2.4", "5"]).default(vec!["2.4", "5"]).try_into_definition().unwrap();

    assert_eq!(
        materialized_parameter(&definition, ParameterSource::Operator, Value::from(vec!["2.4"]),).unwrap().value,
        Value::Array(vec![Value::from("2.4")])
    );
    assert!(matches!(
        materialized_parameter(&definition, ParameterSource::Operator, Value::from(vec!["6"]),),
        Err(dokime_runtime::ApiError::Parameter(ParameterResolverError::InvalidChoice { .. }))
    ));

    let optional = Parameter::list("bands", "Bands", ValueType::Text).optional().default(None::<String>).try_into_definition().unwrap();
    assert_eq!(resolve_parameter_spaces(&[optional], &|_| None).unwrap()["bands"].values, vec![Value::Null]);
}

#[test]
fn struct_declaration() {
    let declared = Soak::declaration().into_iter().map(|parameter| parameter.try_into_definition().unwrap()).collect::<Vec<_>>();
    let by_name = |name: &str| {
        declared
            .iter()
            .find(|definition| definition.metadata.identifier.as_str() == name)
            .unwrap_or_else(|| panic!("{name} was not declared"))
            .clone()
    };

    // The field's name is the id, its type is the value type, and the rest is
    // what the attribute carried.
    let target = by_name("target_c");
    assert_eq!(target.value_type, ValueType::Number);
    assert_eq!(target.min, Some(-50.0));
    assert_eq!(target.max, Some(200.0));
    assert_eq!(target.unit.as_ref().map(|unit| unit.symbol.as_str()), Some("°C"));
    assert!(target.required);
    assert_eq!(target.expression, ParameterExpr::Literal { value: Value::Number(60.0) });
    // A field with no label is read by its own name, titled.
    assert_eq!(by_name("hold_samples").metadata.label, "Hold Samples");
    assert_eq!(by_name("hold_samples").value_type, ValueType::Integer);
    // `Option<T>` is the optional form; `Vec<T>` is the list form.
    assert!(!by_name("operator").required);
    assert_eq!(by_name("ports").value_type, ValueType::Array);
    // A choice declares the closed set it admits and the label of each value.
    let sweep = by_name("sweep");
    assert_eq!(sweep.selection.as_ref().map(|selection| selection.choices.clone()), Some(vec![Value::from("fast"), Value::from("slow")]));
    assert_eq!(sweep.expansion, dokime_runtime::definition::ParameterExpansion::Single);
    assert_eq!(
        sweep.extensions.get("choice_labels"),
        Some(&Value::Object(BTreeMap::from([
            ("fast".to_owned(), Value::from("Fast")),
            ("slow".to_owned(), Value::from("Slow and careful")),
        ])))
    );
}

#[test]
fn same_struct_decodes() {
    let decoded = Soak::from_parameters(&case()).unwrap();

    assert_eq!(decoded.target_c, 60.0);
    assert_eq!(decoded.hold_samples, 5);
    assert_eq!(decoded.operator, None);
    assert_eq!(decoded.ports, vec!["/dev/ttyUSB0".to_owned()]);
    assert_eq!(decoded.sweep, Sweep::Fast);
}

#[test]
fn value_refuses_name() {
    for (name, value, said) in [
        ("target_c", Value::from("warm"), "declared Number"),
        ("target_c", Value::Number(1_000.0), "highest it may be is 200"),
        ("hold_samples", Value::from("five"), "declared Integer"),
        ("hold_samples", Value::Integer(0), "lowest it may be is 1"),
        ("ports", Value::from("not-a-list"), "is not a list"),
        ("sweep", Value::from("sideways"), "is not one of: fast, slow"),
    ] {
        let mut values = case();
        values.insert(name.to_owned(), value.clone());
        let refused = Soak::from_parameters(&values).err().unwrap_or_else(|| panic!("{name} accepted {value:?}"));

        assert!(matches!(&refused, dokime_runtime::Error::Parameter { name: field, .. } if field == name), "{refused}");
        assert!(refused.to_string().contains(said), "{refused}");
    }
    // A run that never resolved a declared input is refused too: the case and
    // the declaration disagreeing is not the same as an absent value.
    let mut values = case();
    values.remove("operator");
    assert!(Soak::from_parameters(&values).is_err());
}

#[test]
fn list_choice_multiple() {
    let declared = Sweeps::declaration().into_iter().map(|parameter| parameter.try_into_definition().unwrap()).collect::<Vec<_>>();

    assert_eq!(declared[0].value_type, ValueType::Array);
    // A list of a choice is the same closed set, more than one of it: the list
    // is what says "more than one", and the resolver restricts every element by
    // the choices beside it.
    assert_eq!(declared[0].list, Some(ParameterList { element_type: ValueType::Text, min_items: Some(1), max_items: None }));
    let selection = declared[0].selection.as_ref().expect("a closed set");
    assert_eq!(selection.choices, vec![Value::from("fast"), Value::from("slow")]);
    // The optional single choice is the same set, one of it, and may be null.
    assert_eq!(declared[1].value_type, ValueType::Text);
    assert_eq!(declared[1].expansion, dokime_runtime::definition::ParameterExpansion::Single);
    assert!(!declared[1].required);

    let values = BTreeMap::from([
        ("sweeps".to_owned(), Value::Array(vec![Value::from("slow"), Value::from("fast")])),
        ("fallback".to_owned(), Value::Null),
    ]);
    let decoded = Sweeps::from_parameters(&values).unwrap();
    assert_eq!(decoded.sweeps, vec![Sweep::Slow, Sweep::Fast]);
    assert_eq!(decoded.fallback, None);

    // A list shorter than its declaration allows is the host's refusal, made
    // against the declaration before anything runs.
    let definition = Sweeps::declaration().into_iter().next().unwrap().try_into_definition().unwrap();
    assert!(matches!(
        materialized_parameter(&definition, ParameterSource::Operator, Value::Array(Vec::new())),
        Err(dokime_runtime::ApiError::Parameter(ParameterResolverError::TooFewItems { .. }))
    ));
}

#[test]
fn field_handle_references() {
    assert_eq!(Soak::target_c().name(), "target_c");
    assert_eq!(DataExpression::from(Soak::sweep()), dokime_runtime::expressions::parameter("sweep"));
}

#[test]
fn choices_round_fields() {
    use dokime_runtime::records::DeclaredRecord;
    use dokime_runtime::telemetry::TelemetryRow;

    let details = SweepSelected { sweep: Sweep::Slow }.into_details();
    assert_eq!(details["sweep"], Value::from("slow"));
    assert_eq!(SweepSelected::from_details(&details).unwrap().sweep, Sweep::Slow);

    let row = SweepSample { sweep: Sweep::Fast }.into_row();
    assert_eq!(SweepSample::from_row(&row).unwrap().sweep, Sweep::Fast);
    let unknown = BTreeMap::from([("sweep".to_owned(), Value::from("unknown"))]);
    assert_eq!(SweepSample::from_row(&unknown).unwrap_err().field, "sweep");
    assert!(SweepSelected::from_details(&unknown).is_err());
}

#[test]
fn numeric_bounds_params() {
    let scalar = Parameter::number("temperature", "Temperature").min(-50.0).max(200.0).default(60.0).try_into_definition().unwrap();
    for value in [-50.0, 200.0] {
        assert!(materialized_parameter(&scalar, ParameterSource::Operator, Value::Number(value)).is_ok());
    }
    assert!(matches!(
        materialized_parameter(&scalar, ParameterSource::Operator, Value::Number(201.0)),
        Err(dokime_runtime::ApiError::Parameter(ParameterResolverError::OutOfBounds { .. }))
    ));
    let list = Parameter::list("holds", "Holds", ValueType::Integer).min(1.0).max(5.0).default(vec![1, 5]).try_into_definition().unwrap();
    assert!(resolve_parameter_spaces(std::slice::from_ref(&list), &|_| None).is_ok());
    assert!(matches!(
        materialized_parameter(&list, ParameterSource::Operator, Value::Array(vec![Value::Integer(1), Value::Integer(6)])),
        Err(dokime_runtime::ApiError::Parameter(ParameterResolverError::OutOfBounds { .. }))
    ));
}

#[test]
fn macro_expansion_groups() {
    let definitions = ExpansionInputs::declaration().into_iter().map(|parameter| parameter.try_into_definition().unwrap()).collect();
    let output = materialize_template_cases(
        "fixture.registration",
        &template(definitions),
        Default::default(),
        dokime_runtime::definition::CaseMaterializationOptions {
            max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
            force: false,
            registration_id: dokime_runtime::model::identity::new_id(),
            first_label_number: 1,
        },
    )
    .unwrap();
    assert_eq!(output.case_count, 8);
    for case in output.test_cases {
        let input = ExpansionInputs::from_parameters(&case.parameters).unwrap();
        assert!(input.fixed);
        assert_eq!(input.names, vec!["A", "B"]);
        assert_eq!(input.y, input.x * 10);
        assert!(input.batches == vec!["A", "B"] || input.batches == vec!["C"]);
        assert_eq!(case.parameters["enabled"], Value::Bool(input.enabled));
    }
}
