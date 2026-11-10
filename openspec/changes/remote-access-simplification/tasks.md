# Remote Access Simplification Tasks

## 1. Contract

- [ ] 1.1 Settle the open points in `proposal.md` with the owner from the mechanism inventory, and record which mechanisms stay.
- [ ] 1.2 Lock the source inventory: clients, command-line tool, desktop, server listener, tests and guidance.

## 2. Implementation

- [ ] 2.1 Remove grants and per-operation permissions for clients.
- [ ] 2.2 Read the connection from environment variables in every client and SDK, with the pairing file as the same values in a file.
- [ ] 2.3 Remove the mechanisms task 1.1 retires, with their tests and guidance.

## 3. Final Validation

- [ ] 3.1 Run focused client, server and desktop connection tests, and the remote control story from a second machine.

## Workflow Follow-Up

- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
