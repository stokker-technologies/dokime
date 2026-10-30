# One-Click Remote Access Files

## Why

Remote access currently requires choosing an endpoint and naming each pairing.
Exporting one secret file should discover the host's endpoints; importing it
should securely connect without network configuration in the WebView.

## What Changes

- Replace `.dokime-pair` and its single endpoint with exact JSON v1 fields:
  `version`, `endpoints`, `backend_identifier`, `label`, `ca_certificate_pem`, `access_key`.
- Export through one Remote Access action, confirming listener enablement when
  disabled, generating a readable label and retaining independent revocation.
- Detect and rank hostname, physical/private and other interface addresses;
  exclude virtual/VPN interfaces and mark loopback as same-PC only.
- Persist a CA and reissue listener leaves under it as detected names change.
- Import through the native picker or window drop, save credentials in the OS
  keyring, create a profile and connect using staggered parallel verified attempts.
- Remember the winning endpoint. Share the Rust connector between GUI and CLI;
  provide equivalent Python behavior, retaining `DokimeClient.from_pairing`.
- Add an NSIS private-network inbound application rule and update affected guides.

## Impact

Clean development break: no old file or security-store compatibility, storage v1.
Invalid existing storage is preserved and requires a fresh root. Files remain
secret, bounded to 1 MiB and strictly validated. No raw key crosses the WebView.
Owner authorizes small Conventional Commits locally only, no push, release,
installer build, umbrella lane or CI simulation. Validation is check-only plus
the requested focused deterministic tests, overriding normal delivery proof.
