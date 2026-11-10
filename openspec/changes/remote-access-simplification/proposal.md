# Remote Access Simplification

Status: direction agreed with the owner on 2026-11-10; the scope is not
locked. The inventory of today's mechanisms is under research and the
security level is still under discussion. Independent of `grpc-plugins`.

## Why

Connecting a client to a backend involves pairing files, a profile registry,
a credential store, a pinned certificate authority, backend identity checks,
access keys and grants. Remote control from a second machine is essential;
most of that machinery is not.

## Agreed

- Every connected client may call every Public operation. There are no
  grants, scopes or per-operation permissions. The network and the people on
  it are trusted, and misuse is the user's responsibility.
- A client is configured through environment variables. No client parses a
  `.env` file itself; mise or any other tool supplies the environment.

```sh
DOKIME_ENDPOINT=http://bench-pc:7443
```

```python
client = dokime.connect()                       # reads the environment
client = dokime.connect("bench.dokime-remote")  # or a pairing file with the same values
```

## Open Points

1. Authentication and transport security. Options under discussion:

   | Level | Client Supplies | Protects Against |
   | --- | --- | --- |
   | None | An endpoint | Nothing; anyone who reaches the port can run executions |
   | Shared key | An endpoint and one access key | Connecting to the wrong backend by mistake; casual access |
   | Shared key and pinned certificate | Endpoint, key, fingerprint | Also reading or altering traffic on the network |

2. Which of today's mechanisms stay: the profile registry, the credential
   store, the certificate authority, fallback endpoints, the stable listener
   and direct links. Decided once the inventory and sizes are known.
3. Whether the server keeps its own `.env` loader or reads only the
   environment like the clients.

## Impact

- Clients, the command-line tool, the desktop connection screen and the
  server's remote listener.
- Existing pairing files and saved profiles may stop working; the development
  storage policy allows that.
