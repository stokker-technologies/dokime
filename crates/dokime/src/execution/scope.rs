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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ExecutionScope {
    #[row(description = "The host's token for the execution attempt.")]
    pub execution: String,
    #[row(
        description = "The stage inside it: a step or criterion id, or empty for the run itself."
    )]
    pub stage: String,
    #[row(description = "The operation being served, or empty where the run itself is the call.")]
    pub route: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdempotencyKey(String);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::operations::ContextValue for ExecutionScope {
    const KEY: &'static str = "dokime.execution.scope";
}

impl ExecutionScope {
    pub fn execution(execution: impl Into<String>) -> Self {
        Self {
            execution: execution.into(),
            stage: String::new(),
            route: String::new(),
        }
    }

    #[must_use]
    pub fn staged(mut self, stage: impl Into<String>) -> Self {
        self.stage = stage.into();
        self
    }

    #[must_use]
    pub fn routed(mut self, route: impl Into<String>) -> Self {
        self.route = route.into();
        self
    }

    #[cfg(feature = "native")]
    pub fn current(host: &harmos::plugin::sidecar::Supervisor) -> Option<Self> {
        host.context().value::<Self>().ok().flatten()
    }

    #[cfg(feature = "native")]
    pub fn serving<O: harmos::operations::Declares>(
        host: &harmos::plugin::sidecar::Supervisor,
    ) -> Option<Self> {
        Self::current(host).map(|scope| {
            scope.routed(<O::Declaration as harmos::operations::OperationDeclaration>::ID)
        })
    }

    pub fn key(&self) -> IdempotencyKey {
        IdempotencyKey(format!(
            "{}:{}{}:{}{}:{}",
            self.execution.len(),
            self.execution,
            self.stage.len(),
            self.stage,
            self.route.len(),
            self.route,
        ))
    }
}

impl IdempotencyKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for IdempotencyKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for IdempotencyKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
