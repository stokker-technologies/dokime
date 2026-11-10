# Privacy and Data Flows

Dokime records engineering evidence on the backend machine. The current first-party
application has no vendor usage-analytics or automatic crash-report upload path.
Here, “telemetry” means measurements stored for the operator, including host
measurements; it does not mean analytics sent to Stokker Technologies. Installed
plugins can implement their own networking and must be reviewed separately.

## Storage Locations and Access

In the table, `A` is the application folder and `D` is the data folder. The default
distribution identifier `I` is `com.stokker.dokime`; an organization distribution
can choose another identifier.

| Platform | `A` | `D` | `L` (Logs) |
| --- | --- | --- | --- |
| Windows | `%LOCALAPPDATA%/I` | `A/data` | `A/logs` |
| Linux | `${XDG_DATA_HOME:-$HOME/.local/share}/I` | `A/data` | `${XDG_STATE_HOME:-$HOME/.local/state}/I/logs` |
| Other Unix builds | Same XDG/home convention as Linux; no separate macOS Library convention | `A/data` | Same XDG state convention as Linux |

`--application-root` or `DOKIME_APPLICATION_ROOT` overrides `A` and puts logs at
`A/logs`. `DOKIME_OUTPUT_ROOT` can override `D` when no explicit application-root
option suppresses ambient overrides. Explicit SDK/runtime configuration can
select another store. Inspect the actual backend's paths, especially when
using a remote backend; the desktop's user folder is not the remote store.
These conventions are owned by `crates/dokime-runtime/src/paths.rs`.

“Local readers” below means the account running the backend, processes with that
account's access, administrators and anyone granted filesystem access. Trusted
native plugins run with the user's OS authority; process separation is not a
filesystem sandbox. Authorized connected clients can read evidence through the
backend API. Dokime does not encrypt evidence at rest: the customer must enable
operating-system disk encryption and configure file permissions and backups.

## Data Inventory

The path symbols expand per OS using the preceding table. Export destinations
are chosen by the operator and may be network or synchronized folders.

| Data Item | Source | Stored Path Pattern | Who Can Read It | How It Leaves the Machine |
| --- | --- | --- | --- | --- |
| Execution identity, device/bench identifiers, parameters, status, timestamps and operator notes/responses | Procedures, operators and plugins | `D/local.sqlite` and its journal/WAL files | Local readers; authorized clients | Reports, API/SDK exports, data-root backups; plugins can transmit their own inputs |
| Host and instrument measurements | Host collectors and installed plugins | `D/telemetry/*.arrow`, indexed by `D/local.sqlite` | Local readers; authorized clients | Reports, evidence exports, API reads and backups; plugin-defined endpoints |
| Photos, screenshots, audio and captured files | Operator capture, cameras, microphones, filesystem and plugin capture sources | `D/files/<sha256>`, metadata in `D/local.sqlite` | Local readers; authorized clients | Report artifacts, file/evidence exports and backups; plugins may transmit captured content |
| Application preferences, plugin settings and secret availability metadata | Operator settings and plugin discovery | `D/local.sqlite` | Local readers; authorized settings clients | Support bundle settings copies and backups; plugin receives its configured settings |
| Protected secret values and client credentials | Operator secret entry and remote pairing | OS protected credential store; catalog metadata in `D/local.sqlite` | OS-authorized application and secret-consuming plugins | Used for configured authentication; protected values are not collected into support bundles |
| Backend profiles and access files | Pairing/import and remote configuration | Windows `%APPDATA%/dokime/backends.json`; Linux `${XDG_CONFIG_HOME:-$HOME/.config}/dokime/backends.json`; macOS `$HOME/Library/Application Support/dokime/backends.json`; `DOKIME_PROFILES_FILE` can override; exported `.dokime-remote` at chosen destination | Local readers and recipients of exported access files | Explicit pairing-file sharing; copying application configuration |
| Operational logs, errors and free-form diagnostic messages | Backend and plugin logging | `L/<distribution-slug>.jsonl` and log rotation files | Local readers; authorized diagnostic clients | Selected log tail in support ZIP; manual log sharing or machine backups |
| Reports, exports and desktop cached views | Operator report/export actions | Chosen destination; application cache under `A/cache` | Filesystem-authorized readers and recipients | Explicit sharing, synchronized folders and external backup systems |
| Data-root backup ZIP | Desktop backup/reset workflow | By default a sibling `data-backup-YYYYMMDDTHHMMSSZ.zip`; selected destination can differ | Filesystem-authorized readers and recipients | Explicit copying/sharing or external backup systems; contains unredacted data-root contents |
| Support ZIP | Explicit diagnostics download | Temporary archive during assembly; chosen download destination | Requesting client and filesystem-authorized readers/recipients | Operator explicitly sends the ZIP; collection does not submit it to the vendor |

## Collection and Network Flows

Procedures and operators supply notes, responses, parameters and media. Plugins
publish measurements to the backend. The runtime stores structured evidence and
settings in SQLite, measurements in compressed Arrow IPC files and captured
bytes in content-addressed files. Reports and finite exports read saved evidence;
connected clients request those results from the selected backend. Backups copy
the data root rather than creating an anonymized report.

The host schema in `plugins/sidecars/host/src/domain/telemetry/table.rs` includes
`host_name`, `user`, `git_user_name`, `git_user_email`, OS/kernel/CPU identity and
load, memory, uptime, temperature, battery identity (including
`battery_serial_number`) and power-plan information. Network rows include
`primary_interface`, `adapter_id`, `address`, traffic/error counters and internet
state. Wi-Fi rows include `wifi_adapter`, `wifi_ssid`, `wifi_bssid`,
`wifi_profile`, authentication/cipher/security, channel/band, signal, rates,
nearby scan results and congestion. Bluetooth rows include
`bluetooth_adapter`, `bluetooth_device_names`, radio state and device counts.
Probe status/error text, sample indices and timestamps accompany measurements.
Availability depends on the platform and enabled collectors. These fields can
identify employees, machines, locations or nearby devices even without notes.

Update checks resolve the configured release feed (GitHub for the standard
distribution), request release metadata/`latest.json`, and download selected
updates. Plugin installation/update flows can also download release assets.
Those services receive ordinary request metadata such as the source IP address;
this is distinct from uploading evidence. Organization feeds may use other hosts.
The host plugin's latency collector sends ICMP echo probes to its configured
latency host, default `1.1.1.1`; `wifi_latency_host`, `wifi_latency_ms` and
`wifi_packet_loss_percent` are stored locally. Instrument plugins, capture URLs
and explicitly configured remote backends introduce their own endpoints.
Review those endpoints and plugin code for each deployment. There is no claim
that Dokime or third-party plugins make no network requests.

## Support Bundles

The server collector in `crates/dokime-server/src/grpc/backend.rs` selects backend
identity, health, the settings frame, installed-plugin settings and a bounded
tail of `<distribution-slug>.jsonl`. It does not collect the evidence database, Arrow streams
or media files. Free-form settings and messages can still repeat evidence or
personal information. Secret declarations expose identifiers and availability,
not protected credential values. `MANIFEST.txt` lists each member and its possible
data categories; `manifest.json` records sizes and availability, including absent
log tails. Settings copies redact Git identity fields, email-containing strings
and user-profile paths. Log tails omit whole lines containing those patterns;
ordinary diagnostic bytes remain unchanged. Windows, Linux and macOS profile
path patterns and the backend account's home path are recognized.

Redaction is enabled by default. An operator who needs personal diagnostic data
can set `DOKIME_SUPPORT_INCLUDE_PERSONAL_DATA=1` in the backend process environment
before starting it; unset it and restart to restore redaction. This does not
include protected secret values. It is not a desktop preferences toggle.
Redaction is conservative pattern matching, not anonymization: unlabeled names,
nonstandard paths, hostnames and other free-form content may remain. Review every
bundle before sharing, and use an approved support channel.

## Retention and Deletion

Archiving an execution hides it from ordinary history; it does not erase its
evidence. There is no scheduled evidence retention and no per-execution deletion
control. A plot's visible time range does not change retention.

The desktop can back up and reset its local data root after stopping the backend
and obtaining the storage lease. Backup walks regular files under that root,
skips symbolic links and the root lock file, and writes an unencrypted ZIP.
Reset deletes the data-root contents; it is not selective and is not secure
media erasure. Existing storage is never silently reset. A fresh root leaves the
old root intact. See `crates/dokime-gui/src-tauri/src/native_session/storage.rs`.

Reset does not remove sibling backup ZIPs, exported reports/access files, copies
on remote machines, OS credential-store entries, or application-root caches and
logs outside `D`. The customer owns deletion of those copies and backup retention.
Decisions still required from the product owner are scheduled retention,
per-execution deletion and a coordinated deletion workflow covering exported
copies; none is provided by this document.

## Article 30 Worksheet

Copy this table into the customer's processing record and complete the final
column for each deployment. It supplies product facts, not a completed compliance
assessment. The [Data Protection Commission's Article 30 guidance](https://www.dataprotection.ie/en/dpc-guidance/records-of-processing-article-30-guidance)
explains the processing-record requirement and accountability purpose.

| Record Field | Dokime Facts to Reuse | Customer Must Complete |
| --- | --- | --- |
| Controller and Contacts | Dokime is software deployed by the customer; installing it does not identify the controller | Legal entity, joint controllers if any, representative and DPO/contact details |
| Purposes | Execution of engineering tests, evidence review, reporting and troubleshooting | Actual approved purposes, employment context and limits on reuse |
| Data Subjects | Operators and people identified in notes, media, account, machine or nearby-device information may be represented | Actual subject categories, including employees, contractors or bystanders |
| Data Categories | Inventory above; plugin-specific data can extend it | Enabled collectors/plugins, actual fields and classifications, any sensitive data |
| Recipients | Authorized clients, chosen report/backup/support recipients and configured plugin endpoints | Internal roles, service providers and external recipient categories |
| International Transfers | No automatic vendor evidence upload; feed requests and chosen exports/endpoints can cross borders | Countries, organizations, transfer arrangements and applicable safeguards |
| Erasure Time Limits | No scheduled retention or per-execution deletion; whole-root reset leaves external copies | Time limits per category, deletion owner/process and backup/export handling |
| Security Measures | OS credential store for protected secrets; remote TLS/authentication where configured; local storage is not application-encrypted | Disk encryption, access permissions, endpoint/plugin approval, backup protection, incident process and verification |
| Record Ownership | Configuration and plugin changes can change this inventory | Record owner, deployment/version, review date and review triggers |

If acting as a processor, maintain the applicable processor record with customer
controllers, processing categories and deployment-specific transfers and measures;
the customer determines its legal role and applicable obligations.
