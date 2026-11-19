# Dokime Python SDK

`dokime-sdk` provides typed plugin declarations and async clients for a running
Dokime host. It requires Python 3.13 or later. Resources close explicitly.

## Public Homes

```python
from dokime_sdk import Plugin, ResultStatus, Suite, connect
from dokime_sdk.operations import Context, Visibility, operation
from dokime_sdk.templates import TestTemplate, criterion, record, step
```

The operation and template modules each own a distinct `Context`. Import a
context from its module when authoring that kind of handler. Plugin handlers
reach shared plugin state through `ctx.plugin`.

## Starter

[The starter](examples/starter/) contains one uv project with an installed
`src/starter_project/` package. `dokime new <name> --lang python` substitutes the
project name and its normalized Python package name. There are no root entry
scripts, argument parsing or shipped lockfile. The user's first `uv sync`
creates the lock.

```sh
uv sync
uv run python -m unittest
uv run python -m starter_project.plugin
uv run python -m starter_project
```

The plugin module prints `harmos listening <host>:<port>` as the first stdout
line, flushes it, and serves until interrupted. The plugin version comes from its
owning installed distribution, including editable installs; a class `version`
overrides that metadata. Source-only plugins must declare a class version.
The package module connects to the host,
registers the plugin, creates a cycle, materializes its template classes and runs
each execution. It prints progress and sealed outcomes, seals the cycle and
writes its evidence archive under `_output/`. Answer operator questions in the
desktop. `mise run test` checks the declared suites, template, steps, criterion
and operations without starting a product host.

Set `DOKIME_REMOTE` to an HTTP origin or `.pair` file in the process environment
or working-directory `.env`. Set `DOKIME_PLUGIN_ADDRESS` to override the plugin
listener's default `127.0.0.1:0`. Explicit arguments take precedence, then the
process environment, then `.env`, then defaults. Keep secrets in Secret Storage.

## Bound Client Flow

```python
client = await connect(connect_timeout=30.0)
try:
    await client.plugins.register(ExamplePlugin())
    cycle = await client.cycle(label="Capture")
    cases = await client.cases.materialize(ConfirmationTestTemplate)
    for case in cases:
        execution = await cycle.execution(case)
        async for progress in execution.run():
            print(progress.phase)
        outcome = await execution.seal()
        print(outcome.status == ResultStatus.PASSED)
    await cycle.seal()
    evidence = await cycle.evidence()
finally:
    await client.close()
```

`execution.start()` and `execution.watch()` also remain available separately.
Closing a client unregisters its own registrations and closes its connection.
Execution sealing and sealed progress require the corresponding host contract;
the SDK reports unsupported contracts explicitly.

## Examples and Repository Proof

- [Starter](examples/starter/): confirmation suite, public echo operation and client flow.
- [Thermal](examples/thermal/): parameterized procedure, records, observers and window sampling.
- [Client](examples/client/src/client_example/__main__.py): browse bound cycle handles.

```sh
mise -C sdk/python run prepare
mise -C sdk/python run format
mise -C sdk/python run check
mise -C sdk/python run test -- --layer unit
mise -C sdk/python run example-test
```

The check task verifies formatting, lint, types, strict authored/generated stubs
and the three intentional public homes. Generated wire modules remain private
and tracked. Regenerate through the generation tasks; never edit them by hand.
Repository host stories live in `tests/system/`; their shared host support lives
in `tests/support/`. They require prepared host fixtures and are not shipped in
the starter. Installed-wheel and cross-language stories remain repository proof.
