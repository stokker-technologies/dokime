# Remote Access Simplification Tasks

## 1. Contract

- [x] 1.1 Recheck the mechanism inventory against the current tree before removing anything.
- [x] 1.2 Lock the source inventory: clients, command-line tool, desktop, server listener, tests and guidance.

## 2. Implementation

- [x] 2.1 Give a connected client every operation the backend serves to clients, per the scope task 1.1 settles.
- [x] 2.2 Read explicitly named or environment-selected pairing files in existing Rust, desktop and Python client readers.
- [x] 2.3 Remove the mechanisms the scope retires, with their tests and guidance, and rename the pairing file extension to `.pair`.
- [ ] 2.4 Make every client reconnect by itself: repeat reads, resume watches without gaps or duplicates, and repeat state-changing requests under their request identity.
- [ ] 2.5 Apply the pairing-file connection contract to the new pure Python, C# and C++ client halves with `grpc-plugins` SDK migration.

## 3. Final Validation

- [ ] 3.1 Run focused client, server and desktop connection tests, and the remote control story from a second machine.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
