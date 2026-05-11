# persona-system skill

Work here when the change concerns OS/window-manager abstractions, focus
events, input-buffer observations, target identity, or backend adapters.

Rules for work here:

- Model observations as typed pushed events.
- Keep backend handles inside data-bearing adapter objects.
- Keep routing decisions in `persona-router`.
- Keep terminal PTY byte transport in `persona-terminal`.
- Use `niri msg --json windows` for current-state focus probes and
  `niri msg --json event-stream` for pushed focus changes. Filter by tracked
  `NiriWindow` id before emitting Persona observations.
- Keep live subscription state in the Kameo `FocusTracker`. Do not bypass
  the actor mailbox when turning event-stream rows into observations.
- Escalate if a backend cannot push the needed event; do not add polling as a
  fallback.
