# ADR: Git remote operations security and recovery boundary

## Status

Accepted — 2026-07-26

## Context

Remote Git operations combine repository mutation, network transport, host identity and
credential access. Mdir4 must not collect or persist credentials, silently trust a new SSH
host key, or hide a partially completed repository operation behind a generic error.

This decision is the G3-00 gate. It applies before Fetch, Pull, Push, remote editing or Clone
is implemented.

## Decisions

### Credentials and authentication

- Mdir4 never accepts, stores, logs, renders or serializes a password, personal access token,
  private key passphrase, cookie or credential-bearing remote URL.
- Authentication is delegated only to the operating system's configured Git credential helper,
  SSH agent, and Git/SSH command-line integration. There is no application credential prompt.
- A remote URL containing a user-info component is rejected before a remote configuration write.
- Errors and progress are redacted before they cross the transport backend boundary. The original
  process command, environment and stderr are never retained in application state.

### Host identity and TLS

- SSH uses the user's `known_hosts` policy. An unknown or changed host key is never accepted by
  Mdir4. Git/SSH may present its normal terminal interaction only outside the TUI transport flow;
  otherwise the operation fails with an actionable message.
- HTTPS certificate/TLS validation is delegated to Git's transport. Certificate errors fail the
  operation; there is no insecure retry or "trust once" control.

### Remote mutation policy

- Force push is not exposed. Push always uses Git's normal non-force semantics.
- Pull uses `--ff-only`. A diverged branch is reported instead of creating an implicit merge or
  rebase. The existing explicit Branch-screen Rebase command remains the user-controlled rebase
  path.
- Fetch does not modify the worktree or index. Pull, Push, Clone, remote configuration changes
  and conflict continue/abort are mutations and participate in the common mutation lease.

### Transport lifecycle, cancellation and cleanup

Every transport operation reports one of these phases:

```text
Queued → Resolving → Authenticating → Transferring → Applying → Terminal
```

- The Git Transport lane is a single bounded worker (capacity 16) with a non-blocking submit
  path. A second active G3 mutation fails immediately as `Busy`; it is not queued behind the
  first operation.
- Fetch/Pull/Push/Clone may resolve and authenticate without the common mutation lease. They
  acquire it non-blockingly immediately before their first transfer, keep it through terminal
  cleanup, and perform no transfer when it is unavailable.
- Cancellation and deadlines are out-of-band control handles. A terminal result is emitted
  exactly once for success, failure, cancellation, timeout and panic.
- Clone creates a target directory only after resolve/auth succeeds. On cancellation or failure,
  Mdir4 removes only a partial target directory it created for that operation; it never removes a
  pre-existing target.
- A rebase, merge or cherry-pick conflict is not auto-resolved. The UI exposes the conflict state
  and explicit continue/abort actions in the later G3-08 workflow.

## Backend capability contract

The future transport backend must provide:

- remote and tracking metadata without network access;
- redacted typed failures for helper/agent unavailable, host-key, TLS, authentication, network,
  cancellation and timeout cases;
- phase/progress notifications capped at 20 Hz;
- an out-of-band cancellation hook and deadline-aware process/network execution;
- test-only fake transport support for every terminal and phase transition.

A backend that requires raw credential callbacks, automatically approves host keys, cannot be
cancelled, or cannot redact errors is not eligible.

## Recovery guidance

- Before retrying a failed Fetch/Push, refresh remote metadata and show the redacted failure.
- After a failed Pull or Rebase, preserve the repository's Git state and direct the user to the
  explicit conflict workflow; Mdir4 must not run reset, clean, force checkout or force push.
- After an interrupted Clone, retain existing targets and clean only the operation-owned partial
  directory described above.

## Consequences

Remote support may initially fail where a system credential helper or SSH agent requires an
interactive prompt. This is deliberate: secure OS-managed authentication is preferred to a TUI
secret prompt. G3-01 may implement cached metadata without opening a network connection; Fetch
and later operations require the transport lane and fake-contract tests first.
