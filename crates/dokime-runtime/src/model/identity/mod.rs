/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Runtime Creation Of Domain Values.

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn new_id<T>() -> T
where
    T: TryFrom<String>,
    T::Error: std::fmt::Debug,
{
    T::try_from(uuid::Uuid::now_v7().to_string())
        .expect("a freshly generated UUID is a valid Dokime identity")
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn fresh_identities_unique() {
        let identifiers = std::thread::scope(|scope| {
            let workers = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        (0..10_000)
                            .map(|_| super::new_id::<dokime_contract::core::ArtifactId>())
                            .collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>();
            workers
                .into_iter()
                .flat_map(|worker| worker.join().unwrap())
                .collect::<Vec<_>>()
        });
        let mut unique = std::collections::BTreeSet::new();
        for identifier in identifiers {
            assert_eq!(identifier.as_str().as_bytes()[14], b'7');
            assert!(unique.insert(identifier), "fresh identity was reused");
        }
    }
}
