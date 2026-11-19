## 1. Starter Source

- [ ] 1.1 Add `--from <checkout>` and `DOKIME_NEW_FROM` to `dokime new` for Python, Rust, C# and C++; local-path SDK dependency per language; actionable errors for a missing starter.
- [ ] 1.2 Switch every story that creates a starter to the checkout under test and delete the hand-edited SDK overrides.
- [ ] 1.3 Remove the C++ default's dependency on a published archive from every test.

## 2. Lanes

- [ ] 2.1 Adopt the library revision that carries plugin lanes.
- [ ] 2.2 Expose lane declaration and per-operation assignment in the Python SDK, with declaration-time refusals.
- [ ] 2.3 Expose the same in the Rust, C# and C++ SDKs, idiomatic to each.
- [ ] 2.4 Prove serial and independent lanes against the product host in every language with deterministic stories.

## 3. Validation and Delivery

- [ ] 3.1 Focused proof of every scenario.
- [ ] 3.2 Hosted validation after delivery.
