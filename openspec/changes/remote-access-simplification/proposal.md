# Remote Access Simplification

Status: direction agreed with the owner on 2026-11-10; the scope below is
proposed and not locked. Independent of `grpc-plugins`.

## Why

Remote control from a second machine is essential. Today it works through a
pairing file, a profile registry, an operating-system credential store, a
key per client with revocation, session history, a certificate authority
with reissue, backend identity negotiation and endpoint racing. A connected
client already has full access; the complexity is in managing connections,
not in permissions.

## Agreed

- Every connected client may call every operation the backend serves to
  clients. The network and the people on it are trusted, and misuse is the
  user's responsibility.
- A client is configured through the environment. No client parses a `.env`
  file itself; mise or any other tool supplies the environment.

## Proposed Scope

The pairing file is the only connection input. It carries the endpoints, one
access key and the server certificate.

```sh
DOKIME_REMOTE=./bench.dokime-remote
```

```python
client = dokime.connect()                       # reads DOKIME_REMOTE
client = dokime.connect("bench.dokime-remote")  # or names the file
```

| Mechanism Today | Proposed |
| --- | --- |
| Pairing file | Stays; the single source for a connection |
| Access key and encrypted transport | Stay: one key per backend and one self-signed server certificate, both in the pairing file |
| Remote listener and the local loopback listener | Stay |
| Profile registry (`backends.json`) and `dokime backend` commands | Replaced by a folder of pairing files; listing known backends is listing that folder |
| Operating-system credential store and secret modes | Removed; the key lives in the pairing file |
| A key per client, labels and per-client revocation | Removed; exporting again replaces the one key |
| Session history | Removed |
| Certificate authority with reissue | Replaced by the one certificate; changing it means exporting again |
| Racing several endpoints | Replaced by trying the listed endpoints in order |
| Operations refused to remote clients | Removed; a remote client can do what a local one can |

About 2,400 to 2,800 lines of source and 1,200 to 1,600 of tests serve the
removed mechanisms today.

What is given up: revoking one machine without affecting the others,
protected storage of the key, a record of who connected, and keeping a
pairing valid across a certificate change.

## Open Points

1. Confirm the proposed scope, row by row.
2. Whether the check that a client reached the backend it expected stays. It
   catches a pairing file used against the wrong machine after an address
   changes hands.
3. Whether the server keeps its own `.env` loader or reads only the
   environment like the clients.

## Impact

- Clients, the command-line tool, the desktop connection screen and the
  server's remote listener.
- Existing pairing files and saved profiles stop working; the development
  storage policy allows that.
