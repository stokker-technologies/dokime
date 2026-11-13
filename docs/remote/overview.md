# Remote Access

Start the current host with an explicit storage root and loopback bootstrap
endpoint. Use the composed library Access services to configure a separate TLS
listener and create a pairing for each remote client. See
[Host Serving](../execution-host.md#serve-client-grpc) for the exact contracts.

## Pair and Connect

Read Access.Listener.Get, then Set the whole writable listener value with its
expected revision. Access.Pairings.Create supplies an endpoint, pinned certificate
and bearer credential. Clients install that trust and send the credential through
their configured gRPC channel. Retain pairing files as credentials.
Access.Pairings.List returns metadata; Delete revokes one pairing. Certificate
rotation revokes pairings and requires fresh trust. Product services share the
same authorization layer as composed library services.

## Operate

Use an SDK client to admit a managed executable bundle or remote plugin URL,
materialize cases, create/start executions, answer prompts and inspect evidence.
The [CLI Guide](../execution-host.md#command-line-candidate) lists currently
available commands. Pairing-file CLI convenience remains an explicit parity task.

## Storage and Recovery

Remote clients never open the host's data directory. An occupied root is refused
before database access; restart reads retain sealed evidence and frozen schemas.
Stop and Cancel preserve their distinct scheduling and cleanup behavior.
Losing observation does not cancel admitted work. See [Privacy](../privacy.md).
