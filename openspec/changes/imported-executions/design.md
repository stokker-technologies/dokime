# Imported Executions Design Notes

## Scope and Ownership

The locked design and protobuf sketch are in proposal.md. Implementation work
is inventoried only in tasks.md. This authoring task changes specifications and
Markdown references, not runtime, SDK source or generated bindings.

`dokime-protocol` owns generated origin definitions and light helpers.
`dokime-operations` owns import declarations, mappings, policy and handlers;
`dokime-runtime` owns lifecycle, transaction publication and recovery.
`dokime-storage` owns persisted execution/evidence shapes and columnar storage,
without a callback into runtime or report rendering. SDKs depend on protocol,
never host/storage. Existing report owners stage output for the transaction.

## Python Lifecycle Follow-Up Inventory

The reference rewrite establishes flat target APIs. These source APIs still
expose context managers and are deliberately unchanged in this task:

| Source and Stub | Existing API | Required Flat Surface |
| --- | --- | --- |
| `sdk/python/python/dokime/backend.py`, `backend.pyi` | Backend.__aenter__/__aexit__; startup exists only on entry | Construct Backend, await start(), await close(); gRPC still removes this backend in its own wave. |
| `sdk/python/python/dokime/client/client.py`, `client.pyi` | DokimeClient.__aenter__/__aexit__ | Existing connect()/close() are sufficient for clients. |
| `sdk/python/python/dokime/client/client.py`, `client.pyi` | new_cycle() via @asynccontextmanager | Use existing create_cycle() with an explicit public cycle close(). |
| `sdk/python/python/dokime/client/execution/cycle.py`, `cycle.pyi` | Cycle.__aenter__/__aexit__; _close() is private | Public close() must seal and drain, preserve cancellation shielding and expose cleanup failures; caller try/finally preserves cleanup on failure. |
| `sdk/python/python/dokime/client/query/reader.py`, `reader.pyi` | FiniteReader.__aenter__/__aexit__ | Existing aclose() suffices for readers. |

Guides use explicit close calls and retain `aclose()` for existing readers.
Backend.start() and Cycle.close() in rewritten references are proposed public
entry points, not claims about the current SDK. Do not call magic/private
methods as a workaround. The gRPC SDK example uses ordinary serve/connect calls
and explicit close; imported values themselves own no open server resources.

Current `openspec/specs/python-backend` and `python-client` prescribe context
managers. They are outside this task's edit whitelist, so replacement deltas are
under this change rather than editing canonical requirements directly. Archived
reference text is updated where permitted; it is not evidence that source was
changed or revalidated. `grpc-plugins` retains ownership of backend removal.

Root README.md was searched and contains backend/client/reader context examples.
The authoring edit whitelist excludes it; pending explicit expansion of that
whitelist, its rewrite remains a guide follow-up alongside canonical spec folding.
