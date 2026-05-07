# persona-system — architecture

*Portable OS, window-manager, focus, and input-observation boundary.*

`persona-system` names what Persona needs from the operating system without
forcing router, harness, or store code to know about Niri, Wayland, macOS, or
any other backend.

---

## 0 · TL;DR

This repo owns system observations as pushed events. It does not decide routing
policy and it does not move terminal bytes.

```mermaid
flowchart LR
    "Niri backend" -->|"focus events"| "SystemAdapter"
    "input recognizer" -->|"buffer events"| "SystemAdapter"
    "SystemAdapter" -->|"SystemEvent Frame"| "persona-router"
    "SystemAdapter" -->|"observation records"| "persona-store"
```

## 1 · Component Surface

`persona-system` exposes:

- typed target identity for windows, panes, and harness surfaces;
- focus-state observations;
- prompt/input-buffer observations;
- event subscription surfaces for consumers;
- backend adapter traits or data-bearing adapter objects.

## 2 · State and Ownership

The component owns observations and subscriptions. Backend adapters may keep
backend-specific handles, sockets, or registration state. Durable observation
history is committed through `persona-store` when the assembled runtime needs
it.

## 3 · Boundaries

This repo owns:

- portable system target types;
- pushed focus/input event surfaces;
- backend abstraction for Niri and later OS ports.

This repo does not own:

- delivery decisions (`persona-router`);
- harness lifecycle (`persona-harness`);
- terminal PTY transport (`persona-wezterm`);
- shared frame definitions (`persona-signal`);
- durable transaction ordering (`persona-store`).

## 4 · Invariants

- Producers push events; consumers subscribe.
- Backend-specific details stay behind data-bearing adapter objects.
- The router receives observations and decides policy.
- Unknown system state is explicit typed state, not a reason to poll.

## Code Map

```text
src/target.rs  portable target identity
src/event.rs   focus/input observation records
tests/         smoke tests for typed observations
```

## See Also

- `../persona-router/ARCHITECTURE.md`
- `../persona-harness/ARCHITECTURE.md`
- `../persona-signal/ARCHITECTURE.md`
