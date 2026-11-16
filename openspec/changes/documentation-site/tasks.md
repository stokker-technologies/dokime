# Tasks

Proposed on 2026-11-13; phase one implemented on 2026-11-16. The SDK references follow the SDK
restructures and the examples.

## 0. Decisions Before Implementation

- [x] 0.1 Decided by the owner on 2026-11-14: an entry is a release line showing its latest patch; `latest` points at the newest stable line and `dev` at the development branch.
- [x] 0.2 Decided by the owner on 2026-11-16: the generator is Starlight; the Docusaurus trial is dropped.
- [x] 0.3 Verify the reference generator for each language is current (recorded in design.md on 2026-11-16).

## 1. Site and Guides

- [x] 1.1 Add the site project over `docs/` with static search and the release selector.
- [x] 1.2 Add `docs:build` and `docs:serve`.

## 2. References

- [x] 2.1 Generate the operations reference from the protocol files.
- [ ] 2.2 Generate the Rust, Python, C# and C++ SDK references after the SDK restructures, confirming each generator handles its SDK as built, and add them to the archive.
- [ ] 2.3 Close doc comment gaps the generators report in each SDK.

## 3. Publication

- [x] 3.1 Build the documentation archive in the release pipeline as a required asset in the index; it holds the guides and the operations reference until 2.2 adds the SDK references.
- [x] 3.2 Build the documentation archive in Nightly as a parallel job that may fail and report it.
- [x] 3.3 Add the pull request documentation build check.

## 4. Site Image

- [x] 4.1 Add `docs:image`: assemble published archives, write the selector entries and `latest`.
- [ ] 4.2 Deploy the image; the deployment target is not chosen yet.

## 5. Final Validation

- [ ] 5.1 Hosted pipelines publish the archive and the assembled site shows `latest`, a release and `dev`.
