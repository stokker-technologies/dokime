# Python SDK

The [Python SDK Guide](../sdk/python/README.md) covers the composed client,
plugin authoring, exact values and runnable examples. Use Python 3.13 or newer,
Git, mise and uv. From the repository root:

```sh
mise -C sdk/python run prepare
mise -C sdk/python run format
mise -C sdk/python run generate -- --check
mise -C sdk/python run build
mise -C sdk/python run install
mise -C sdk/python run check
mise -C sdk/python run unit
mise -C sdk/python run test
mise -C sdk/python run generated
mise -C sdk/python run package
mise -C sdk/python run wheel
mise -C sdk/python run example-test
```

Build exports a pure wheel and source distribution under
`target/build/dev/any/python`. Package verifies those existing bytes and copies
them to `target/package/dev/any/python`. SDK tooling uses the repository root
`.venv`. Generation checking uses temporary output and leaves committed files untouched.
Check includes typing and formatting for the shared Python host fixture and example
story as well as the SDK, examples, tests and tooling. Wheel installs the built
distribution into a fresh environment before running the example's host journey.

The [client](../sdk/python/examples/client/main.py),
[plugin](../sdk/python/examples/plugin/main.py) and
[test](../sdk/python/examples/test/) examples use public modules. The test suite
and explicit template are the command-line starter's authoritative sources.
The CLI owns embedding them. Host-dependent stories require prepared host
fixtures, the controlled clock and explicit credential cleanup. Build those
fixtures through the [execution host tasks](execution-host.md) before running them.
