# Managed Deployment

## Current Release Qualification

The v0.1.2 Windows installer was reported as `NotSigned` by Authenticode in
the enterprise adoption audit. The updater's minisign signature authenticates
updater downloads; it does not supply a Windows publisher identity. No MSI
package is currently delivered. Intune, Configuration Manager and authenticated
corporate proxy deployment still require qualification on the organization's
managed image.

## Certificate Options

Azure Artifact Signing (formerly Trusted Signing) provides managed signing
through Microsoft's [official action](https://github.com/Azure/artifact-signing-action).
An alternative is a code-signing certificate available to SignTool through a
PFX secret. Confirm that the issuer permits export and CI use: a certificate
whose private key must remain in a hardware token cannot simply become a PFX
secret. Private enterprise certificates require deployment of their trust chain
to managed devices.

Signing identifies the publisher and detects modified bytes. It does not grant
SmartScreen reputation. EV certificates no longer bypass SmartScreen reputation
checks; new signed releases can still produce warnings. See Microsoft's
[SmartScreen guidance](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation).
RFC 3161 timestamping and verification belong to the signing operation.

## Signing Secrets

Configure **one** provider using repository or protected-environment secrets:

| Provider | Required Secrets |
| --- | --- |
| Azure Artifact Signing | `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_SIGNING_PROFILE` |
| SignTool with PFX | `WINDOWS_SIGNING_PFX_BASE64`, `WINDOWS_SIGNING_PFX_PASSWORD` |

For Azure, create an Artifact Signing account and a validated certificate
profile, and grant the application identity permission to sign with that
profile (Artifact Signing Certificate Profile Signer). Use the regional
endpoint matching the account. For public distribution, use a public-trust
profile. The action uses service-principal credentials and disables fallback
to an existing Azure CLI login. No OIDC permission is required for this mode.

For PFX, encode the certificate/private-key file as base64 and supply its
nonempty password separately. The Windows SDK must provide `signtool.exe`;
the script checks PATH and then the SDK's x64 tools. The PFX is written only
to a temporary file and removed in a `finally` block. Use an isolated signing
runner and restrict access to signing secrets.

Both modes use SHA-256 file and timestamp digests and RFC 3161 timestamping
at `http://timestamp.acs.microsoft.com` by default. The action's `timestamp-url`
input can select another RFC 3161 service. Every signed executable must return
`Valid` from `Get-AuthenticodeSignature` and contain a timestamp certificate.
Signing or verification failures fail the task. Missing all provider secrets
prints a skip notice and still packages; partial or mixed providers fail.
No Authenticode certificate is currently configured by this change.

The existing `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` remain separate updater credentials.
Stable release policy still requires them. Authenticode runs before minisign,
so the updater signature covers the final signed installer bytes. Packaging
also signs the staged GUI after its icon resources are replaced and signs the
installed CLI before inventory digests are calculated. The standalone exported
executables are signed before their archives are created. The action refreshes
the portable distribution export from those signed product bytes through the
existing exporter, which regenerates its integrity inventory.

### Release Wiring

In the Windows package job, **replace** the existing
`mise run package:rust --profile ...` step with this step, after all Build
artifacts are extracted and before package archiving, checksums, attestations
or upload. Keep the job's existing Windows/release selection condition if it
is more specific. The same step supports unsigned fork and Nightly runs.
Linux retains its existing packaging step. Do not add a later installer
signing step: that would invalidate minisign, SHA256SUMS and attestations.

```yaml
- name: Sign and Package Windows
  if: runner.os == 'Windows'
  uses: ./.github/actions/sign-windows
  env:
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
  with:
    profile: ${{ env.PROFILE }}
    azure-tenant-id: ${{ secrets.AZURE_TENANT_ID }}
    azure-client-id: ${{ secrets.AZURE_CLIENT_ID }}
    azure-client-secret: ${{ secrets.AZURE_CLIENT_SECRET }}
    azure-endpoint: ${{ secrets.AZURE_SIGNING_ENDPOINT }}
    azure-account: ${{ secrets.AZURE_SIGNING_ACCOUNT }}
    azure-profile: ${{ secrets.AZURE_SIGNING_PROFILE }}
    pfx-base64: ${{ secrets.WINDOWS_SIGNING_PFX_BASE64 }}
    pfx-password: ${{ secrets.WINDOWS_SIGNING_PFX_PASSWORD }}
```

The action wraps packaging; credentials are scoped to its steps rather than
written to the job environment. Local packaging can opt into the same hook
with `DOKIME_AUTHENTICODE_PROVIDER=azure` or `pfx` and the corresponding
environment variables mapped in the action. The manifest remains unchanged;
existing `[signing.installer]` certificate-store signing is an alternative
and must not be combined with the hook. Do not sign untrusted pull-request
code with release credentials.

## Windows Installation Scope

The existing installer manifest remains valid. Organization deployment recipes
choose scope through switches; the default is per-user:

```powershell
# Current User; No Elevation Required
Start-Process -FilePath '.\dokime-setup.exe' -ArgumentList '/CURRENTUSER /S' -Wait

# All Users; Run from an Elevated Deployment Agent or Administrator Shell
$setup = Start-Process -FilePath '.\dokime-setup.exe' -ArgumentList '/ALLUSERS /S' -Wait -PassThru
$setup.ExitCode

# Machine Uninstall; Run Elevated
$remove = Start-Process -FilePath "$env:ProgramFiles\com.stokker.dokime\uninstall.exe" -ArgumentList '/S' -Wait -PassThru
$remove.ExitCode
```

Substitute the actual installer filename and distribution identifier. `/S` is
case-sensitive silent mode; `/P` shows progress without the normal wizard;
`/NS` suppresses shortcuts. `/D=<absolute path>` overrides the installation
directory and must be the final argument, without quoting the path inside the
NSIS argument string. `/ALLUSERS` and `/CURRENTUSER` are mutually exclusive.
The organization pack chooses its deployment command; no new manifest setting
or mandatory machine-wide default is introduced.

Per-user setup installs under `%LOCALAPPDATA%\<identifier>` and registers in
HKCU. `/ALLUSERS` uses `%ProgramFiles%\<identifier>`, HKLM, common shortcuts
and machine PATH. Setup and removal require an elevated process for machine
scope; applications do not request elevation. The installer does not add
firewall rules. A deployment agent running as SYSTEM should explicitly pass
`/ALLUSERS`; a default per-user install under SYSTEM is not an interactive
user installation. The machine uninstaller determines scope from its
registered installation location, including when invoked without scope flags.

Exit codes are `0` for success, `1` for user cancellation, `2` for installer
failure, `87` for conflicting scope switches, and `740` when machine scope
requires elevation. Preserve any other nonzero exit as failure. These are
NSIS exits, not MSI's 3010 reboot convention. Signing scripts return zero on
successful signing or the explicit no-secret skip and nonzero on failure.

### Detection and Updates

For Intune Win32 or Configuration Manager application detection, use the
64-bit registry view on the supported x64 Windows target:

```text
Machine: HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall\<identifier>
User:    HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\<identifier>
Default identifier: com.stokker.dokime
Values: DisplayVersion (REG_SZ), InstallLocation (REG_SZ),
        UninstallString (REG_SZ), QuietUninstallString (REG_SZ),
        MachineInstall (REG_DWORD, 1 for machine and 0 for user)
```

Use `DisplayVersion` plus the expected scope and executable presence, not just
the display label. `InstallLocation` is stored quoted. Configure installation
and detection in the same context: machine deployment as SYSTEM/admin, user
deployment as that user. Existing installers predating this change have no
`MachineInstall` marker; detect those per-user versions through HKCU and
`DisplayVersion`. Intune/Configuration Manager qualification must confirm
install, repair, upgrade, cancellation and removal on the managed image.

Upgrade using the same identifier, scope and installation directory; setup
restores the registered directory and replaces packaged application files.
Stop Dokime before managed upgrade. Changing scope is a deliberate uninstall
and reinstall, not a migration; otherwise separate HKCU and HKLM installations
can coexist. Keep machine updates under the deployment agent's control rather
than relying on an unelevated desktop updater.

NSIS provides no transactional rollback guarantee. Retain the approved prior
installer and back up evidence/configuration before upgrading. To roll back,
uninstall the new application in its original scope and reinstall the approved
prior version. Do not select application-data deletion during removal.
Restore compatible evidence only after checking storage-format compatibility;
development versions may require a fresh storage root. Never delete or reset
existing evidence merely to make a downgrade work.

## Linux Package Verification

Download the `.deb` or `.rpm` and `SHA256SUMS` from the same release. Verify the
selected file against its entry before installing:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify ./dokime-package.deb --repo stokker-technologies/dokime
gh attestation verify ./dokime-package.rpm --repo stokker-technologies/dokime
```

Substitute the actual release filenames and run the attestation command only
for the package being deployed. Require successful verification; a missing
attestation is not evidence of provenance. SHA256SUMS detects corruption but
does not independently authenticate its own source. Attestations bind the
downloaded bytes to the publishing repository and workflow.

There is no managed APT or RPM repository in this delivery. Repository signing
is follow-up work: publish APT metadata with signed `InRelease`, distribute the
public key through an explicit `Signed-By` keyring, and sign RPM packages with
`rpmsign` plus signed repository metadata and managed key rotation. The existing
packaging command can explicitly sign individual packages with configured GPG
keys; that capability does not establish a signed public repository.

## Network and Privileges

Dokime normally runs without elevation. The local backend uses a loopback gRPC
listener with an ephemeral port. Do not expose it by adding a broad inbound
firewall rule. The separate remote endpoint is disabled by default; when
enabled, use TLS, authentication, explicit pairing and firewall rules restricted
to the configured interface, port and approved clients. See
[Remote Access](remote/overview.md).

The bundled host plugin's latency probe sends ICMP echo requests to `1.1.1.1`
by default. This is a network measurement, not vendor telemetry. Configure an
approved probe destination or disable the probe where external ICMP is blocked.
ICMP does not traverse an HTTP proxy. Platform ICMP restrictions can make a
probe unavailable without requiring the entire application to run elevated.

The default organization pack uses manual update checks against the Dokime
GitHub release feed. Allow outbound HTTPS to `api.github.com`, `github.com`
and the release asset hosts reached through GitHub redirects when updates are
enabled. Organization packs can select their own update source. Fonts and
organization artwork are bundled locally. Dokime has no vendor telemetry
upload service; plugins and user-authored operations can make their own network
connections and need separate review.

Windows setup also checks for WebView2. If it is absent, the default installer
downloads Microsoft's bootstrapper through `go.microsoft.com` and its download
redirects. Preinstall an approved WebView2 runtime on offline or proxy-restricted
images; the application package alone is not a complete offline prerequisite
bundle. Treat Microsoft runtime servicing as part of the managed-image policy.

Keep loopback connections outside corporate proxies. Remote gRPC requires
end-to-end HTTP/2 and trust in the paired server certificate; TLS interception
must not silently replace that trust. Authenticated proxies, PAC discovery and
managed TLS interception are not qualified deployment guarantees. Validate the
organization's actual proxy policy before enabling online updates or remote
access; offline package distribution remains available.
