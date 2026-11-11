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
use sdk::parameters::ParameterValue;
use sdk::values::FieldValue;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[sdk::choice]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
enum Channel {
    Ch1 = 1,
    Ch36 = 36,

    #[choice(id = "high", label = "Upper Band")]
    Ch149 = 149,
}

#[sdk::choice]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Sweep {
    Fast,
    SlowAndCareful,

    #[choice(id = "http_server")]
    HTTPServer,

    #[choice(label = "Overridden")]
    Ch36,

    #[choice(id = "kept")]
    Unlabelled,
}

#[sdk::choice]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i8)]
enum Offset {
    Below = -1,
    Level = 0,
    Above,
}

/* === Traits === */

/* === Implementations === */

impl Sweep {
    // A Choice Without Discriminants Declares Its Own `value`; The Macro Adds None To Collide With It.
    const fn value(self) -> &'static str {
        "declared"
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn offered<T: ParameterValue>() -> Vec<(String, String)> {
    let choices = T::choices().expect("a choice offers its values");
    choices
        .into_iter()
        .map(|choice| {
            let sdk::Value::Text(id) = choice.value else { panic!("a choice's id is text") };
            (id, choice.label)
        })
        .collect()
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected.iter().map(|(id, label)| ((*id).to_owned(), (*label).to_owned())).collect()
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn discriminants_derive_ids() {
    // A written discriminant is the id; an override wins.
    assert_eq!(offered::<Channel>(), pairs(&[("1", "Ch 1"), ("36", "Ch 36"), ("high", "Upper Band")]));
    assert_eq!(Channel::Ch36.into_value(), sdk::Value::Text("36".to_owned()));
    assert_eq!(Channel::from_value(Some(&sdk::Value::Text("36".to_owned()))).unwrap(), Channel::Ch36);
    assert_eq!(Channel::from_value(Some(&sdk::Value::Text("high".to_owned()))).unwrap(), Channel::Ch149);
    assert!(Channel::from_value(Some(&sdk::Value::Text("ch36".to_owned()))).is_err());
    assert_eq!(<Channel as sdk::prompts::PromptFieldValue>::choices(), ["1", "36", "high"]);
}

#[dokime_tracing::trace_test]
#[test]
fn names_derive_ids() {
    assert_eq!(
        offered::<Sweep>(),
        pairs(&[
            ("fast", "Fast"),
            ("slow_and_careful", "Slow And Careful"),
            ("http_server", "HTTP Server"),
            ("ch36", "Overridden"),
            ("kept", "Unlabelled"),
        ]),
    );
    assert_eq!(Sweep::from_value(Some(&sdk::Value::Text("kept".to_owned()))).unwrap(), Sweep::Unlabelled);
}

#[dokime_tracing::trace_test]
#[test]
fn represented_discriminants_read_back() {
    assert_eq!(Channel::Ch36.value(), 36_u16);
    assert_eq!(Offset::Below.value(), -1_i8);
    assert_eq!(Offset::Above.value(), 1_i8);
    assert_eq!(offered::<Offset>(), pairs(&[("-1", "Below"), ("0", "Level"), ("above", "Above")]));
    assert_eq!(Sweep::Fast.value(), "declared");
}
