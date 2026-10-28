/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::fmt::{self, Write};

use super::super::projection;
use super::{EntryValue, Groups, Scope, ScopeSeed, entry};
use crate::runtime::Result;
use dokime_domain::model::{time::Timestamp, values::Value};

struct HostSeed<'a> {
    scope: Scope<'a>,
    values: &'a std::collections::BTreeMap<String, Value>,
}

struct DurationText(f64);

struct SharedWith<'a>(LeaseRow<'a>);

#[derive(Clone, Copy)]
pub(super) enum LeaseSource<'a> {
    Facts(&'a [crate::telemetry_runtime::LeaseFact]),
    Ledger(crate::telemetry_runtime::leases::LeaseFacts<'a>),
}

#[derive(Clone, Copy)]
enum LeaseRow<'a> {
    Fact(&'a crate::telemetry_runtime::LeaseFact),
    Ledger(crate::telemetry_runtime::leases::LeaseFactRef<'a>),
}

impl fmt::Display for HostSeed<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}|", ScopeSeed(self.scope))?;
        let mut after = None;
        let mut first = true;
        loop {
            let next = projection::HOST_TRACEABILITY_FIELDS
                .iter()
                .filter(|(key, _, _, _)| self.values.get(*key).is_some_and(projection::resource_traceability_value_visible))
                .map(|(_, _, _, source)| *source)
                .filter(|source| after.is_none_or(|after| *source > after))
                .min();
            let Some(source) = next else { break };
            if !first {
                output.write_char('|')?;
            }
            output.write_str(source)?;
            first = false;
            after = Some(source);
        }
        Ok(())
    }
}

impl fmt::Display for DurationText {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_minutes = (self.0.max(0.0) / 60.0).round() as u64;
        let (hours, minutes) = (total_minutes / 60, total_minutes % 60);
        match (hours, minutes) {
            (0, minutes) => write!(output, "{minutes} min"),
            (hours, 0) => write!(output, "{hours} h"),
            (hours, minutes) => write!(output, "{hours} h {minutes} min"),
        }
    }
}

impl<'a> LeaseSource<'a> {
    fn iter(self) -> impl Iterator<Item = LeaseRow<'a>> {
        let (facts, ledger): (&'a [crate::telemetry_runtime::LeaseFact], _) = match self {
            Self::Facts(facts) => (facts, None),
            Self::Ledger(ledger) => (&[], Some(ledger)),
        };
        facts.iter().map(LeaseRow::Fact).chain(ledger.into_iter().flat_map(|ledger| ledger.iter()).map(LeaseRow::Ledger))
    }
}

impl<'a> LeaseRow<'a> {
    fn stream(self) -> &'a dokime_domain::model::identity::Identifier {
        match self {
            Self::Fact(fact) => &fact.stream,
            Self::Ledger(fact) => fact.stream(),
        }
    }
    fn owner(self) -> &'a str {
        match self {
            Self::Fact(fact) => &fact.owner,
            Self::Ledger(fact) => fact.owner(),
        }
    }
    fn opened_at(self) -> &'a Timestamp {
        match self {
            Self::Fact(fact) => &fact.opened_at,
            Self::Ledger(fact) => fact.opened_at(),
        }
    }
    fn closed_at(self) -> Option<&'a Timestamp> {
        match self {
            Self::Fact(fact) => fact.closed_at.as_ref(),
            Self::Ledger(fact) => fact.closed_at(),
        }
    }
    fn gap(self) -> bool {
        match self {
            Self::Fact(fact) => fact.gap,
            Self::Ledger(fact) => fact.gap(),
        }
    }
    fn shared_with(self) -> impl Iterator<Item = &'a str> {
        let (facts, ledger): (&'a [String], _) = match self {
            Self::Fact(fact) => (fact.shared_with.as_slice(), None),
            Self::Ledger(fact) => (&[], Some(fact)),
        };
        facts.iter().map(String::as_str).chain(ledger.into_iter().flat_map(|ledger| ledger.shared_with()))
    }
}

impl fmt::Display for SharedWith<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, execution) in self.0.shared_with().enumerate() {
            if index != 0 {
                output.write_str(", ")?;
            }
            output.write_str(execution)?;
        }
        Ok(())
    }
}

pub(in crate::traceability) fn host_override_bytes(key: &str, value: &Value) -> Result<Option<usize>> {
    with_host_value(key, value, |value| match value {
        EntryValue::Text(value) => Ok(Some(super::text_size(value)?.bytes)),
        _ => Ok(None),
    })
}

pub(super) fn host_entries(values: &std::collections::BTreeMap<String, Value>, entries: &mut dyn super::Entries) -> Result<()> {
    for (key, identifier, label, source) in projection::HOST_TRACEABILITY_FIELDS {
        let Some(value) = values.get(*key) else { continue };
        if !projection::resource_traceability_value_visible(value) {
            continue;
        }
        with_host_value(key, value, |value| {
            entry(entries, format_args!("{identifier}"), format_args!("{label}"), value, format_args!("{source}"))
        })?;
    }
    Ok(())
}

pub(super) fn visit_host(
    values: &std::collections::BTreeMap<String, Value>, scope: Scope<'_>, at: Option<&Timestamp>, groups: &mut dyn Groups,
) -> Result<()> {
    groups.group(scope, at, None, format_args!("{}", HostSeed { scope, values }), false, &mut |entries| host_entries(values, entries))
}

pub(super) fn visit_leases(facts: LeaseSource<'_>, scope: Scope<'_>, at: Option<&Timestamp>, groups: &mut dyn Groups) -> Result<()> {
    groups.group(scope, at, None, format_args!("{}|traceability.leases", ScopeSeed(scope)), false, &mut |entries| {
        for lease in facts.iter() {
            let stream = lease.stream();
            entry(
                entries,
                format_args!("{stream}.owner"),
                format_args!("{stream} owner"),
                EntryValue::Text(format_args!("{}", lease.owner())),
                format_args!("traceability.leases"),
            )?;
            entry(
                entries,
                format_args!("{stream}.leased_from"),
                format_args!("{stream} leased from"),
                EntryValue::Text(format_args!("{}", lease.opened_at())),
                format_args!("traceability.leases"),
            )?;
            match lease.closed_at() {
                Some(closed) => entry(
                    entries,
                    format_args!("{stream}.leased_until"),
                    format_args!("{stream} leased until"),
                    EntryValue::Text(format_args!("{closed}")),
                    format_args!("traceability.leases"),
                )?,
                None => entry(
                    entries,
                    format_args!("{stream}.leased_until"),
                    format_args!("{stream} leased until"),
                    EntryValue::Text(format_args!("open")),
                    format_args!("traceability.leases"),
                )?,
            }
            if lease.shared_with().next().is_some() {
                entry(
                    entries,
                    format_args!("{stream}.shared_with"),
                    format_args!("{stream} shared with"),
                    EntryValue::Text(format_args!("{}", SharedWith(lease))),
                    format_args!("traceability.leases"),
                )?;
            }
            if lease.gap() {
                entry(
                    entries,
                    format_args!("{stream}.gap"),
                    format_args!("{stream} gap"),
                    EntryValue::Text(format_args!(
                        "the stream's owner restarted while this run held it, so rows are missing for that interval"
                    )),
                    format_args!("traceability.leases"),
                )?;
            }
        }
        Ok(())
    })
}

fn with_host_value<T>(key: &str, value: &Value, read: impl for<'a> FnOnce(EntryValue<'a>) -> Result<T>) -> Result<T> {
    match (key, value) {
        ("memory_total_bytes" | "memory_used_bytes", Value::Integer(bytes)) => {
            read(EntryValue::Text(format_args!("{:.2} GB", *bytes as f64 / 1_000_000_000.0)))
        }
        ("system_uptime_s", Value::Number(seconds)) => read(EntryValue::Text(format_args!("{}", DurationText(*seconds)))),
        ("system_uptime_s", Value::Integer(seconds)) => read(EntryValue::Text(format_args!("{}", DurationText(*seconds as f64)))),
        _ => read(EntryValue::Borrowed(value)),
    }
}
