# persona-system — architecture

*Portable OS, window-manager, focus, and input-observation boundary.*

`persona-system` names what Persona needs from the operating system without
forcing router or harness code to know about Niri, Wayland, macOS, or any other
backend.

---

## 0 · TL;DR

This repo owns system observations as pushed events. It does not decide routing
policy and it does not move terminal bytes.

```mermaid
flowchart LR
    "Niri backend" -->|"focus events"| "FocusTracker"
    "input recognizer" -->|"buffer events"| "SystemAdapter"
    "FocusTracker" -->|"observation frame"| "signal-persona-system"
    "SystemAdapter" -->|"observation frame"| "signal-persona-system"
    "signal-persona-system" -->|"pushed observation"| "persona-router"
```

## 1 · Component Surface

`persona-system` exposes:

- typed target identity for windows, panes, and harness surfaces;
- focus-state observations;
- a `system` CLI for one-shot focus probes and focus subscriptions;
- a Niri focus source backed by `niri msg --json windows` and
  `niri msg --json event-stream`;
- a Kameo `FocusTracker` that owns subscription focus state while the event
  stream is running;
- prompt/input-buffer observations;
- event subscription surfaces for consumers;
- backend adapter traits or data-bearing adapter objects.

## 2 · State and Ownership

The component owns observations and subscriptions. Backend adapters may keep
backend-specific handles, sockets, or registration state. A live Niri
subscription keeps `FocusTracker` as the data-bearing actor; compositor events
enter through that mailbox before any Persona observation is emitted.

Durable consumer history is not owned here; consumers that need history persist
it through their own Sema database. If `persona-system` later needs durable
subscription registrations, backend cursors, or adapter state, it owns a
system-scoped Sema database for that state rather than writing into another
component's database.

## 3 · Boundaries

This repo owns:

- portable system target types;
- pushed focus/input event surfaces;
- backend abstraction for Niri and later OS ports.

This repo does not own:

- delivery decisions (`persona-router`);
- harness lifecycle (`persona-harness`);
- terminal PTY transport (`persona-wezterm`);
- system frame definitions (`signal-persona-system`);
- durable transaction ordering for consumers.
- any other component's Sema database.

## 4 · Invariants

- Producers push events; consumers subscribe.
- Backend-specific details stay behind data-bearing adapter objects.
- Live subscription state belongs to Kameo actors, not loose shared objects.
- Niri window id is the first real target key; title, app id, and pid are
  evidence, not identity.
- The router receives observations and decides policy.
- Unknown system state is explicit typed state, not a reason to poll.
- System-owned durability, when present, is limited to subscription/backend
  state and emits observations only after commit.

## Code Map

```text
src/target.rs  portable target identity
src/event.rs   focus/input observation records
src/niri.rs       Niri focus snapshot and event-stream adapter
src/niri_focus.rs Kameo mailbox implementation for `FocusTracker`
src/command.rs    NOTA CLI command surface
tests/                  smoke and actor-runtime constraint tests
```

## See Also

- `../persona-router/ARCHITECTURE.md`
- `../persona-harness/ARCHITECTURE.md`
- `../signal-persona/ARCHITECTURE.md`
- `../signal-persona-system/ARCHITECTURE.md`
