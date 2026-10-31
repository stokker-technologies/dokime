# Remote Access File Design

## Locked Contracts

Owner decisions dated 2026-10-30 are the implementation contract.

```json
{
  "version": 1,
  "endpoints": ["https://lab-host:8765", "https://192.168.1.20:8765"],
  "backend_identifier": "backend-id",
  "label": "Remote 2026-10-30 18-45-00",
  "ca_certificate_pem": "<persistent CA PEM>",
  "access_key": "<per-file secret>"
}
```

Fields are exact, CA/key may be null under existing listener policy. Reject
empty/duplicate endpoint inventories, wildcard origins, mixed TLS policy,
malformed certificates and existing size/secret/identity violations.

## Ownership

- `dokime-domain/model/remote.rs` owns file and operation contracts.
- `dokime-server/grpc/remote/` owns interface discovery, ranking, persistent CA,
  leaf issuance/reissue, listener transitions, per-file keys and revocation.
- `dokime-domain/model/backends.rs` owns ordered profile `endpoints`; its existing
  `endpoint` records the last success (initially the first candidate). Manual lab
  profiles have no alternatives. Credentials bind the profile identifier and expected backend identity; manual
  profiles keep their endpoint association.
- `dokime-client` owns bounded file import, keyring transactions and the verified
  candidate race. `ClientPreferences` owns connection stagger configuration.
- Native desktop code owns confirmation, picker/drop imports, file publication
  and connection. The WebView sees no raw secret. Paired Clients permits rename.
- CLI imports with `backend add --remote`; Python retains `from_pairing(path)`.
- Installer packaging never manages firewall rules. The desktop's optional host
  action owns private-network access under the installation capability.

## Certificates And Discovery

Current source persists only a self-signed leaf, not a CA. Replace that identity
with a persistent CA certificate/key and a separately signed leaf/key. Reissue
under the same CA on startup, export and periodic interface changes. Persist
before replacing the listener, preserving existing secrets and storage on errors.
Specific binds advertise their address. Wildcard binds advertise hostname first,
private physical addresses before public addresses in the listener address family;
exclude virtual/VPN adapters,
link-local, multicast and unspecified addresses. Loopback fallback is same-PC.
All exported names are included in the active leaf SANs.

## Connection And Persistence

Try the remembered candidate first, then file order, with configured stagger.
Only a completed TLS and backend identity negotiation can win. Cancel losing
attempts. A wrong CA or identity fails that candidate and cannot win the race.
Save the winner only if the profile still matches the selected snapshot.
File imports retain transactional profile/keyring rollback and independent keys.

## Validation Boundary

The owner requests workspace check/format, TypeScript check/lint, Python
lint/typecheck and focused tests only. Final delivery is local atomic commits;
leave the change available for review, with no publication or release work.

## Provider Protection Boundary

The extra `stkr project check` provider audit finds no required status checks on
GitHub `dev`, with no ruleset supplying them. The declared protection is strict
`CI / Required`, bound to GitHub Actions (app 15368). Restoring remote protection
requires owner authorization beyond this local-only handback. Preserve this
repair obligation; do not treat the unavailable provider proof as passed.
