# Remote Access Simplification Tasks

## 1. Contract

- [ ] 1.1 Recheck the mechanism inventory against the current tree before removing anything.
- [ ] 1.2 Lock the source inventory: clients, command-line tool, desktop, server listener, tests and guidance.

## 2. Implementation

- [ ] 2.1 Give a connected client every operation the backend serves to clients, per the scope task 1.1 settles.
- [ ] 2.2 Read the pairing file named explicitly or through the environment in every client and SDK.
- [ ] 2.3 Remove the mechanisms the scope retires, with their tests and guidance, and rename the pairing file extension to `.pair`.
- [ ] 2.4 Make every client reconnect by itself: repeat reads, resume watches without gaps or duplicates, and repeat state-changing requests under their request identity.

## 3. Final Validation

- [ ] 3.1 Run focused client, server and desktop connection tests, and the remote control story from a second machine.

## Workflow Follow-Up

- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
