# persona-system — architecture

*Portable OS, window-manager, and focus-observation boundary.*

`persona-system` names what Persona needs from the operating system without
forcing router or harness code to know about Niri, Wayland, macOS, or any other
backend.

> **Scope.** Any "sema" reference here means today's `sema` library
> (rename pending → `sema-db`). The eventual `Sema` is broader;
> today's persona-system is a realization step on the
> eventually-self-hosting stack (eventually the OS itself is in Sema,
> at which point this OS-boundary layer goes away). See
> `~/primary/ESSENCE.md` §"Today and eventually".

---

## 0 · TL;DR

This repo owns system observations as pushed events and privileged OS actions
as a separately-gated surface. It does not decide routing policy and it does
not move terminal bytes.

```mermaid
flowchart LR
    "Niri backend" -->|"focus events"| "FocusTracker"
    "FocusTracker" -->|"observation frame"| "signal-persona-system"
    "signal-persona-system" -->|"pushed observation"| "persona-router"
```

## 1 · Component Surface

`persona-system` exposes:

- typed target identity for windows, panes, and harness surfaces;
- focus-state observations;
- a `system` CLI for one-shot focus probes and focus subscriptions;
- a `persona-system-daemon` socket skeleton for the first Persona stack;
- a Niri focus source backed by `niri msg --json windows` and
  `niri msg --json event-stream`;
- a Kameo `FocusTracker` that owns subscription focus state while the event
  stream is running;
- privileged action records such as force-focus and focus-drift suppression;
- event subscription surfaces for consumers;
- backend adapter traits or data-bearing adapter objects.

## 1.5 · Supervision-relation reception (skeleton mode)

persona-system is **deferred** — its focus-tracker work pauses
until a real consumer surfaces. But the daemon must still come up as a
supervised first-stack component so the prototype's "all six daemons
ready" witness can pass.

In **skeleton mode**, the daemon:

1. Reads its `signal-persona::SpawnEnvelope` at startup; binds
   `system.sock` at mode 0600 by applying the `PERSONA_SOCKET_MODE`
   value from the Persona spawn envelope.
2. Answers `signal-persona::SupervisionRequest` from a `SupervisionPhase`
   actor — `ComponentReady { component_started_at }` once the socket is
   bound; `ComponentHealthReport { health: Running }`.
3. Returns `SystemReply::SystemRequestUnimplemented` for every domain
   request (focus subscription, focus snapshot, system status query) —
   the contract decodes the variant, the daemon answers honestly that
   the behavior is not built in this wave.

The Niri backend, FocusTracker, and privileged-action surfaces stay as
existing design but do not run in skeleton mode. They activate when the
deferral ends.

## 2 · State and Ownership

The component owns observations and subscriptions. Backend adapters may keep
backend-specific handles, sockets, or registration state. A live Niri
subscription keeps `FocusTracker` as the data-bearing actor; compositor events
enter through that mailbox before any Persona observation is emitted.

Read-only observations and privileged actions are separate surfaces. Focus
state is an observation that consumers subscribe to. Force-focus and
focus-drift suppression are privileged actions; they require manager-created
system authority. A non-privileged connection may observe permitted state but
cannot request an OS-level action.

Prompt cleanliness, typed write leases, and programmatic write injection are not
system observations in the current stack. They are terminal transport facts
owned by `persona-terminal` and `terminal-cell` through the
`signal-persona-terminal` contract.

Durable consumer history is not owned here; consumers that need history persist
it through their own Sema database. If `persona-system` later needs durable
subscription registrations, backend cursors, or adapter state, it owns a
system-scoped Sema database for that state rather than writing into another
component's database.

## 3 · Boundaries

This repo owns:

- system runtime behavior for portable targets defined by
  `signal-persona-system`;
- pushed focus event surfaces;
- backend abstraction for Niri and later OS ports.

This repo does not own:

- delivery decisions (`persona-router`);
- harness lifecycle (`persona-harness`);
- terminal PTY transport (`persona-terminal`);
- system frame definitions (`signal-persona-system`);
- terminal prompt and input-gate contracts (`signal-persona-terminal`);
- durable transaction ordering for consumers.
- any other component's Sema database.

`signal-persona-system` owns the contract types, their rkyv wire derives, and
their NOTA text derives. `persona-system` consumes those records directly; it
does not define local mirror records for `SystemTarget`, `FocusObservation`,
or `SystemRequest`.

## 4 · Invariants

- Producers push events; consumers subscribe.
- The daemon accepts the `signal-persona-system` frame boundary.
- The daemon applies the managed spawn-envelope socket mode to `system.sock`
  before accepting client traffic.
- The daemon answers `SystemStatusQuery` with typed health and readiness.
- A recognized but unbuilt daemon request returns
  `SystemRequestUnimplemented`; it must not hang or print an untyped text
  error.
- `persona-system` must not duplicate contract-owned records.
- Backend-specific details stay behind data-bearing adapter objects.
- Privileged actions are not observations; they require the persona daemon's
  system connection class.
- Live subscription state belongs to Kameo actors, not loose shared objects.
- Niri window id is the first real target key; title, app id, and pid are
  evidence, not identity.
- The router receives observations and decides policy.
- Unknown system state is explicit typed state, not a reason to poll.
- System-owned durability, when present, is limited to subscription/backend
  state and emits observations only after commit.
- Prompt cleanliness is terminal-owned, not system-owned.

## Code Map

```text
src/command.rs     NOTA CLI command surface over `SystemRequest`
src/daemon.rs      socket daemon skeleton over `signal-persona-system::Frame`
src/event.rs       local focus-state helpers only
src/niri.rs        Niri focus snapshot and event-stream adapter
src/niri_focus.rs  Kameo mailbox implementation for `FocusTracker`
src/target.rs      local harness-to-system-target helper
tests/             smoke, daemon, and actor-runtime constraint tests
```

## Constraint Witnesses

| Constraint | Nix-visible witness |
|---|---|
| The daemon applies the managed spawn-envelope socket mode. | `checks.<system>.system-daemon-applies-spawn-envelope-socket-mode` |
| The daemon answers typed health/readiness. | `checks.<system>.system-daemon-answers-status-readiness` |
| The daemon returns typed unimplemented for unfinished recognized requests. | `checks.<system>.system-daemon-returns-typed-unimplemented` |
| Niri subscriptions pass events through the Kameo mailbox. | `tests/actor_runtime_truth.rs` |
| `persona-system` does not own terminal prompt-gate vocabulary. | `tests/smoke.rs` |

## See Also

- `../persona-router/ARCHITECTURE.md`
- `../persona-harness/ARCHITECTURE.md`
- `../signal-persona/ARCHITECTURE.md`
- `../signal-persona-system/ARCHITECTURE.md`
