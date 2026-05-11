# persona-system

Portable system boundary for Persona.

This crate defines typed contracts for:

- harness window identity;
- focused-window state;
- prompt/input-buffer observations;
- delivery decisions made from pushed system events.

The first implementation target is the current Niri-based Persona OS stack.

The `system` CLI accepts one NOTA command:

```sh
system '(ObserveFocus (NiriWindow 198))'
system '(FocusSubscription (NiriWindow 198))'
```

`ObserveFocus` reads `niri msg --json windows` once. `FocusSubscription` emits
an initial `FocusObservation` and then follows `niri msg --json event-stream`,
filtering noisy compositor events through the Kameo `FocusTracker` actor that
owns the tracked window state.
