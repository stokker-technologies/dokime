# Use Python Remotely

Install the pure `dokime` wheel with Python 3.13 or newer. It provides remote
Harmos plugin serving, typed Dokime catalogs, composed documents and generated
async client stubs. See the [SDK guide](../../sdk/python/README.md).

```sh
uv add /path/to/dokime_sdk-0.1.5-py3-none-any.whl
```

## Serve a Plugin

The [starter](../../sdk/python/examples/starter/) is the source embedded by
`dokime new bench --lang python`. Start Dokime separately, then serve the Python
plugin and add its printed URL to the host:

```sh
dokime new bench --lang python
cd bench
uv add /path/to/dokime_sdk-0.1.5-py3-none-any.whl
mise run test
mise run serve
```

The starter's test uses a prepared product host with a controlled clock. Set
`DOKIME_TEST_HOST` to the host fixture executable before `mise run test`. The
story answers the operator prompt, reads the retained confirmation record and
checks the evaluated outcome and plugin cleanup.

## Connect a Client

```python
import dokime

client = await dokime.connect("http://127.0.0.1:8765")
try:
    # Generated stubs are keyed by fully qualified service name.
    print(client.stubs.keys())
finally:
    await client.close()
```

Use `await dokime.connect("lab.pair")` to require the paired backend identity,
or supply HTTPS, a pinned `certificate_pem` and `access_key` explicitly.
Identity verification requires the generated backend service method; the SDK
refuses the pairing connection when that definition is absent.

## Query Retained Evidence

Repository `proto/dokime/v1` currently contains messages but no client service
definitions. Host operations, template expansion, cycle start/watch, and finite
evidence clients need those authoritative definitions before a real-host client
journey can run. The native `DokimeClient`, backend launcher, replay readers and
dataframe adapters have been removed. Their examples are superseded by the
generated client surface, not retained as compatibility APIs.

The [host query contract](../query-api.md) still describes finite reads and
continuations. The [SDK validation guide](../../sdk/python/README.md#validate)
records the exact host capabilities needed to run and seal the shared suite.
