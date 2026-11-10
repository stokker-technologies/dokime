# Remote Access Simplification Tasks

## 1. Contract

- [x] 1.1 Recheck the mechanism inventory against the current tree before removing anything.
- [x] 1.2 Lock the source inventory: clients, command-line tool, desktop, server listener, tests and guidance.

## 2. Implementation

- [x] 2.1 Give a connected client every operation the backend serves to clients, per the scope task 1.1 settles.
- [x] 2.2 Read the pairing file named explicitly or through the environment in every client and SDK.
- [x] 2.3 Remove the mechanisms the scope retires, with their tests and guidance, and rename the pairing file extension to `.pair`.
- [ ] 2.4 Make every client reconnect by itself: repeat reads, resume watches without gaps or duplicates, and repeat state-changing requests under their request identity.

## 3. Final Validation

- [ ] 3.1 Run focused client, server and desktop connection tests, and the remote control story from a second machine.

## Workflow Follow-Up

- Source ownership: remote and backend models and operation declarations; `dokime-client` transport, pairing and profiles; `dokime-server` remote access and authorization; runtime query revocation consumers; CLI connection and backend commands; desktop connection, native-session consumers and binding generation; their tests, Python pairing readers, examples and affected documentation. Distribution branding and certificate subject labels remain with the organization-pack worker.
- Connection recovery requires server request deduplication for ordinary mutations and replay cursors for watches that currently have none; execution command identities alone do not cover the requirement. The pinned Harmos middleware exposes metadata and status, not request/response frames or a replay owner. Meeting 2.4 for product and plugin operations requires a coupled Harmos transport change; that upstream scope decision remains with the owner.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
