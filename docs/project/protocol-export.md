# Protocol Contracts

`dokime proto export --out proto` and `dokime-proto-<version>.zip` contain
Dokime and Harmos contracts, their complete Google Protocol Buffers import closure,
and license texts under `licenses/`. Google sources come from the pinned protoc
package (Protocol Buffers 36.2); their upstream license is retained in
`licenses/Protobuf.txt`. Dokime and Harmos use `licenses/Apache-2.0.txt`.

Run these commands from the exported or extracted directory. No remote schema
registry or additional proto include directory is required.

## Protoc

Compile the complete contract inventory with the Protocol Buffers compiler:

```sh
find dokime harmos -name '*.proto' -print > inputs.txt
protoc --proto_path=. --include_imports --descriptor_set_out=contracts.bin @inputs.txt
```

On PowerShell, create the input list with:

```powershell
Get-ChildItem dokime,harmos -Recurse -Filter *.proto |
    ForEach-Object { [IO.Path]::GetRelativePath((Get-Location).Path, $_.FullName).Replace('\', '/') } |
    Set-Content -Encoding utf8 inputs.txt
protoc --proto_path=. --include_imports --descriptor_set_out=contracts.bin "@inputs.txt"
```

Add the language-specific output option and gRPC plugin for your consumer.

## Buf

Build the same local schema closure with Buf:

```sh
buf build . --exclude-source-info -o contracts.binpb
```

The export includes a minimal `buf.yaml`. Release packaging refuses missing imports
before creating the archive.
