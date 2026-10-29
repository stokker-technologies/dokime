/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded allocation helpers for wire value projection.

pub(in crate::grpc) fn text(source: &str) -> dokime::harmos::Result<String> {
    let mut target = String::new();
    target.try_reserve_exact(source.len()).map_err(|_| exhausted())?;

    target.push_str(source);
    Ok(target)
}

pub(in crate::grpc) fn slots<T>(length: usize) -> dokime::harmos::Result<Vec<T>> {
    let mut target = Vec::new();
    target.try_reserve_exact(length).map_err(|_| exhausted())?;

    Ok(target)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "canonical projection allocation",
    }
}
