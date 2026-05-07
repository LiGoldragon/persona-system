# Persona System Architecture

`persona-system` is the OS/window-manager abstraction layer for Persona.

```mermaid
flowchart LR
  Router[persona-router] --> Gate[InputGate]
  Gate --> System[persona-system]
  System --> Events[Focus and input events]
  System --> Delivery[Terminal input adapter]
```

The crate should stay small. It names what Persona needs from an operating
system without forcing every downstream component to know about Niri, Wayland,
or a future macOS adapter.
