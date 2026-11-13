# Remote CLI Workflows

Use `dokime-next` from the same candidate as the host. The command-line tool
uses the Rust SDK and the host's final client protocol. See the
[Product Guide](../product-guide.md#interfaces) for output shapes and current
integration limitations, and [Tooling](../tooling.md) for focused builds.

## Export Access

On the host computer, start the loopback bootstrap listener with an explicit
storage root. Keep its terminal open until the host is stopped:

```sh
dokime-next --endpoint 127.0.0.1:7000 server --storage ./bench-data
```

From another local terminal, export a pairing through that bootstrap listener:

```sh
dokime-next --endpoint http://127.0.0.1:7000 access pairing export --label Operator --out operator.dokime-remote --json
```

Export enables the saved authenticated listener when needed. Transfer the
`.dokime-remote` file to the client computer through your chosen secure channel.
It contains ordered endpoints, the expected backend identity, pinned certificate
and bearer credential. Treat the file as a credential. Existing output files
are refused; exporting another pairing does not replace one.

## Connect and Discover

Pass the pairing file directly; the CLI validates its version, suffix and identity.
It tries the saved endpoints with pinned TLS trust and verifies the backend before
using the connection. Local bootstrap URLs use `--endpoint`; authenticated remote
commands use `--remote`:

```sh
dokime-next --remote operator.dokime-remote status --json
dokime-next --remote operator.dokime-remote plugin list --json
dokime-next --remote operator.dokime-remote catalog list --json
dokime-next --remote operator.dokime-remote operation list --json
dokime-next --remote operator.dokime-remote operation show harmos.v1.Status/Get --json
dokime-next --remote operator.dokime-remote operation call harmos.v1.Status/Get --json '{}'
```

`DOKIME_REMOTE` supplies the pairing path when `--remote` is omitted. An explicit
`--endpoint` selects a bootstrap connection instead. `--config FILE` reads the
CLI's TOML settings for connection, operation, watch, message and retained-byte
bounds. Credentials are not printed in successful export summaries or errors.

`catalog list/show` reads the top-level Suites resource. `operation list/show/call`
uses library CatalogEntries.List and the returned descriptors, visibility and
fingerprint. Unary calls accept canonical protobuf JSON through `--json` or
`--input-file`; typed streaming and managed contracts require their typed clients.
Public plugin operations without a direct gRPC binding use Invocations.Invoke.
Product wire identities use `dokime.client.v1`; library identities retain their
library package. Product contracts are currently absent from host discovery,
so raw product calls are blocked until that registration is composed.

## Materialize and Run

Replace the suite and template identifiers below with declarations returned by
`catalog list` and `catalog show SUITE`. Save this CasesCreateRequest as `plan.json`:

```json
{
  "plan": {
    "selections": [{"suite_id": "SUITE", "template_id": "TEMPLATE"}],
    "max_cases": 1
  },
  "request_id": "selection-1"
}
```

Preview validates without registration. Register admits the complete plan and
retains its response for an unchanged request identity:

```sh
dokime-next --remote operator.dokime-remote plan preview --input-file plan.json --json
dokime-next --remote operator.dokime-remote plan register --input-file plan.json --out cases.pb --json
dokime-next --remote operator.dokime-remote cycle create --cases cases.pb --request-id cycle-1 --json
```

`plan materialize --suite SUITE --template TEMPLATE --out cases.pb` also registers
cases. Its optional `--parameters FILE` reads a JSON object mapping parameter names
to arrays of shared Dokime Value messages. It accepts `--request-id` for retries.

Cycle creation starts its frozen origins automatically and returns a `cycle`
response field. Read execution identities from the cycle's execution pages; replace
`CYCLE` and `EXECUTION` below with those returned identities:

```sh
dokime-next --remote operator.dokime-remote execution list --cycle CYCLE --page-size 1 --json
dokime-next --remote operator.dokime-remote execution show EXECUTION --json
dokime-next --remote operator.dokime-remote execution wait EXECUTION --json
```

List prints one response per page and continues with the returned cut, token and
unchanged selection. Execution detail is in `execution`, with its `summary`,
controls, outcome and schema digest. `execution show EXECUTION --schemas --json`
fetches complete descriptor-file pages through Executions.Schemas.Get. Multiple
execution arguments reuse descriptor sets by digest within that command. Automatic
record/stream payload decoding awaits a frozen ID-to-message-name binding.

`cycle append --request FILE` consumes an encoded
`dokime.client.v1.ExecutionsCreateRequest` containing `parent`, explicit execution
bindings and `request_id`. `execution start`, `execution stop`, `cycle rerun` and
`cycle seal` retain their typed operations and accept `--request-id`. Whole-value
metadata changes use Get's value and revision in the final Set request, with
`expected_revision` and a retained request identity; raw product delivery awaits
the discovery registration described above.

## Answer a Prompt

For a running execution that requests operator input:

```sh
dokime-next --remote operator.dokime-remote execution prompt next EXECUTION --json
dokime-next --remote operator.dokime-remote execution prompt show PROMPT --json
dokime-next --remote operator.dokime-remote execution prompt answer EXECUTION --prompt PROMPT --action ACTION --accepted --request-id answer-1 --json
```

Use the prompt identity and declared action returned by the host. Omit `--accepted`
for a negative answer. Without `--prompt`, answer requires exactly one pending
prompt for that execution. Waiting follows progress events and refuses an execution
that finishes without a pending prompt; it does not use polling sleeps.

## Read Reports and Recover

The report commands use Reports.Create for a pure preview and library
Storage.GetFileBytes for bounded ranges:

```sh
dokime-next --remote operator.dokime-remote execution report EXECUTION --out execution.html
dokime-next --remote operator.dokime-remote cycle report CYCLE --out cycle.html
```

They verify the returned cut, total length and SHA-256 before atomically saving
the file, and refuse existing destinations. Rendering or downloading does not
publish a report. The current host does not register Storage.GetFileBytes, so
these downloads remain explicitly unavailable pending library content composition.
Failed downloads leave no final output and preserve existing destinations.

Losing a wait or connection leaves admitted work running. Reconnect to the same
backend, inspect the returned identities and controls, and reuse the same request
identity only for an unchanged mutation. The CLI adds no automatic mutation retry,
cycle sealing or evidence export on interruption. Existing storage roots and
evidence are retained; incompatible development storage requires an explicit
fresh-root or backup/reset decision. See [Remote Access](overview.md).
