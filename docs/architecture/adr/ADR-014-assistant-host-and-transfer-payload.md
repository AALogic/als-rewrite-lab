# ADR-014: Assistant Host And Transfer Payload Boundaries

Status: accepted
Date: 2026-08-05

## Context

QuickCopyAssistant successfully exposes the existing module-018 one-project
copy flow through a compact character surface. ExternalFolderHandoff can then
open WeTransfer and provide the resulting Project directory as a native macOS
folder drag.

The first implementation correctly protects the ALS and Project data, but its
presentation state combines four independent concerns:

```text
copy job lifecycle
character presentation
completed Project payload lifecycle
one provider-specific native drag attempt
```

Adding another destination, another payload action or another character inside
that combined reducer would multiply states and make character presentation an
owner of application behavior. A broad plugin system would be premature and
would introduce executable extension and licensing risks before a second
character exists.

## Decision

Keep the existing domain and application pipeline unchanged. Introduce two
narrow boundaries above module 018 and below the compact UI:

```text
DesktopCopyPreview / DesktopCopyResult
-> QuickCopyJob state
-> TransferPayload state
-> AssistantHost presentation
-> platform adapters
```

### AssistantHost

AssistantHost owns only the compact window's reusable visual structure:

```text
speech and detail presentation
character animation selection supplied by a presenter
contextual controls supplied by the active capability
window movement geometry
separate payload drag geometry
accessible labels
```

It does not parse ALS, plan or execute copies, interpret copy results, choose a
provider, own a native folder path or decide payload lifecycle transitions.
The first implementation hosts exactly one courier experience. Character
catalogs, multiple visible characters and entitlements remain later modules.

### QuickCopyJob

The existing QuickCopy capability retains the one-project state:

```text
destination_required
preparing
ready
working
complete
incomplete
unable
```

It remains the only frontend consumer of module-018 preview and result
contracts. It does not contain provider, handoff or native drag phases.

### TransferPayload

The backend owns the latest eligible completed Project directory as a private
payload candidate. React identifies it only through the originating copy result
identity. React no longer sends an authoritative native path when preparing a
handoff.

Each native drag is a separate one-shot attempt:

```text
payload unavailable
payload available -> attempt armed -> attempt dragging -> attempt consumed
```

The payload candidate remains available after a dropped, cancelled or failed
attempt so the existing explicit provider action can safely create another
attempt. A new preview, new copy, new QuickCopy ALS launch or quick-window
teardown invalidates the payload and every active attempt together.

This refactor does not yet introduce the future visible `delivered` / `reload`
experience. It creates the boundary required to add that behavior deliberately
without changing module 018 or the character host.

### Provider Coordination

Module 026 remains a narrow compatibility coordinator for the current
`wetransfer_web` action:

```text
validate closed provider id
request one attempt from TransferPayload
install the native drag surface
open the backend-owned provider URL
return a path-free prepared contract
```

Provider opening and payload state are separate collaborators even though the
current `W` action invokes both in one user gesture. A provider catalog and
additional destinations are not part of this refactor.

### Platform Interaction

The native adapter consumes a validated payload attempt and exposes one folder
URL with copy-only semantics. AssistantHost supplies distinct geometry for
window movement and payload interaction. The current complete-character drag
image may remain during behavior-preserving migration; changing to a parcel-
only drag image is a later presentation change with separate visual evidence.

## Dependency Direction

```text
AssistantHost
  depends on presentation contracts only

QuickCopy capability
  depends on module-018 IPC contracts
  supplies job presentation facts to AssistantHost

ExternalFolderHandoff
  depends on TransferPayload and platform ports
  never depends on ALS/package internals

TransferPayload
  depends on successful DesktopCopyResult identity and filesystem validation
  never depends on React, provider URLs or browser state
```

## Consequences

- The current courier, copy behavior and WeTransfer entry remain available.
- Copy-job tests no longer need handoff phases to verify copy behavior.
- Payload retry behavior can be tested independently of provider opening.
- React cannot nominate an arbitrary native folder for dragging.
- A later destination catalog can open providers without changing payload
  ownership.
- A later character catalog can replace presentation assets without changing
  capabilities.
- No plugin SDK, OAuth integration, remote upload automation, licensing runtime
  or second character is introduced now.

## Compatibility And Migration

The public ExternalFolderHandoff IPC contract advances to v0.2 because the
redundant `final_target_root` request field is removed. The response and events
remain path-free. Module 025 keeps its visible strings, controls and copy
behavior throughout the migration.

The migration is accepted only when baseline frontend, desktop backend and
workspace tests remain green and the packaged macOS quick flow passes a manual
smoke test.
