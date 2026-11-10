# Remote Access Simplification

Status: locked by the owner on 2026-11-10; implementation has not started
and waits for the owner's go. Independent of `grpc-plugins`.

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

## Scope

The pairing file is the only connection input. It carries the endpoints, one
access key and the server certificate, and its extension is `.pair`.

```sh
DOKIME_REMOTE=./bench.pair
```

```python
client = dokime.connect()                       # reads DOKIME_REMOTE
client = dokime.connect("bench.pair")  # or names the file
```

| Mechanism Today | Decision |
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

## Connection Drops

A transient connection drop is not an error for a client. The client object
stays valid and reconnects by itself. A read is repeated; a watch resumes
where it stopped without gaps or duplicates; a request that changes state
is repeated under the same request identity so the backend applies it
once. The client reports a failure only when the backend stays unreachable
beyond the configured reconnect time.

## Open Points

1. Whether the check that a client reached the backend it expected stays. It
   catches a pairing file used against the wrong machine after an address
   changes hands.
2. Whether the server keeps its own `.env` loader or reads only the
   environment like the clients.

## Impact

- Clients, the command-line tool, the desktop connection screen and the
  server's remote listener.
- Existing pairing files and saved profiles stop working; the development
  storage policy allows that.
