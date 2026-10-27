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
use harmos::sidecar::Refusal;

// Local Imports
use super::{Analysis, Operation, Request};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<A: Analysis> harmos::Dependent for Operation<A> {
    const RESOURCES: &'static [harmos::Resource] = &[];
}

// Each attempt evaluates its own copy of the request on a blocking worker; a
// cancelled attempt detaches the worker, which finishes without an observer.
impl<A: Analysis> harmos::Operation for Operation<A> {
    type Error = Refusal;

    async fn run(self, _: harmos::Context<'_, Self>) -> Result<harmos::Response<super::Response>, Refusal> {
        check::<A>(&self.0)?;
        let request = self.0;
        let result = tokio::task::spawn_blocking(move || Operation::<A>::evaluate(request)).await;
        result.map_err(Refusal::of)?.map(Into::into).map_err(Refusal::of)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn check<A: Analysis>(request: &Request) -> Result<(), Refusal> {
    if request.parameters.len() as u64 > dokime_domain::model::analysis::MAX_PARAMETER_BYTES as u64 {
        return Err(Refusal::of("analysis configuration exceeds its budget"));
    }
    request
        .samples
        .as_ref()
        .ok_or_else(|| Refusal::of("analysis samples are missing"))?
        .check(A::LIMITS.constrained())
        .map_err(Refusal::of)?;
    Ok(())
}

/* === Tests === */
