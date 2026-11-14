# Python SDK

The [Python SDK Guide](../sdk/python/README.md) covers plugin authoring and clients.
Install mise, Python 3.13 or newer, Git and uv, then run from the repository root:

```sh
mise -C sdk/python run prepare
mise -C sdk/python run format
mise -C sdk/python run build
mise -C sdk/python run install
mise -C sdk/python run check
mise -C sdk/python run test
mise -C sdk/python run package
mise -C sdk/python run wheel
```

Build exports one pure wheel and source distribution under
`target/build/dev/any/python`. Package verifies those existing bytes and copies
them to `target/package/dev/any/python`. The installed-wheel story uses an
isolated environment and awaits the product host's conditions. Build the host
fixture through the repository tasks before running host-dependent stories.
All runtime packages include `py.typed`, including generated protocol packages.
