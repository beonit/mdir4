# ADR-006: Remote location read foundation

## Decision

Remote directory paths and entry names are protocol-byte values. They remain separate from
their escaped display strings. The reducer and UI receive typed listing results through a
`RemoteReadBackend`; a fake backend is the first conformance implementation.

The production OpenSSH adapter must implement the same read port. It must not build a remote
shell command, downgrade unknown bytes to a local `PathBuf`, or expose authentication,
host-verification, or endpoint details in UI errors.

## Consequences

- A remote listing is directories-first and byte-name stable, with duplicate names rejected.
- Symlinks are represented distinctly and are never followed implicitly.
- Connection/list/read cancellation and per-location worker ownership are required before the
  production listing adapter is attached to navigation.
- The existing SFTP home probe is discovery only; it does not represent a persistent session.
