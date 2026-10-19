/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod arguments;
mod catalog;
mod facade;
mod listener;
mod message;
mod observers;
mod parameters;
mod plugin;
mod prompt;
mod record;
mod row;
mod settings;
mod suite;
mod template;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use proc_macro::TokenStream;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[proc_macro_attribute]
pub fn parameters(attribute: TokenStream, item: TokenStream) -> TokenStream {
    parameters::parameters(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn prompt(attribute: TokenStream, item: TokenStream) -> TokenStream {
    prompt::prompt(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn choice(attribute: TokenStream, item: TokenStream) -> TokenStream {
    parameters::choice(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn suite(attribute: TokenStream, item: TokenStream) -> TokenStream {
    suite::suite(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn template(attribute: TokenStream, item: TokenStream) -> TokenStream {
    template::template(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn step(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    template::marker("step", item.into()).into()
}

#[proc_macro_attribute]
pub fn criterion(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    template::marker("criterion", item.into()).into()
}

#[proc_macro_attribute]
pub fn observer(attribute: TokenStream, item: TokenStream) -> TokenStream {
    observers::observer(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn catalog(attribute: TokenStream, item: TokenStream) -> TokenStream {
    facade::harmos_attribute("catalog", attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn message(attribute: TokenStream, item: TokenStream) -> TokenStream {
    message::message(message::Kind::Plain, attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn row(attribute: TokenStream, item: TokenStream) -> TokenStream {
    message::message(message::Kind::Row, attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn stream(attribute: TokenStream, item: TokenStream) -> TokenStream {
    observers::stream(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn record(attribute: TokenStream, item: TokenStream) -> TokenStream {
    message::message(message::Kind::Record, attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn listener(attribute: TokenStream, item: TokenStream) -> TokenStream {
    listener::listener(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn plugin(attribute: TokenStream, item: TokenStream) -> TokenStream {
    plugin::plugin(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn operation(attribute: TokenStream, item: TokenStream) -> TokenStream {
    facade::harmos_attribute("operation", attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn section(attribute: TokenStream, item: TokenStream) -> TokenStream {
    settings::section(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn settings(attribute: TokenStream, item: TokenStream) -> TokenStream {
    settings::settings(attribute.into(), item.into()).into()
}

/* === Private Functions === */

/* === Tests === */
