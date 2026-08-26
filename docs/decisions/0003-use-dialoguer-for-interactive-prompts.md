---
type: decision
status: accepted
created: 2026-08-27
supersedes: ["0001"]
superseded_by: []
---

# Use Dialoguer for interactive prompts

## Context
ADR 0001 chose Inquire. Minegr also uses Indicatif for progress output, so its prompt library should fit the same terminal ecosystem.
## Options considered
### Keep Inquire
- Preserves the previous choice.
- Uses a separate terminal UI ecosystem from Indicatif.
### Use Dialoguer
- Provides the prompts Minegr needs and practical official examples.
- Shares the `console-rs` ecosystem with Indicatif.
## Decision
Use Dialoguer for interactive prompts. Continue using Clap for argument parsing and Indicatif for progress output.

This decision supersedes [ADR 0001](0001-Interactive-prompt-crate.md).
## Rationale
Dialoguer meets Minegr's prompt needs and keeps prompts and progress output in the same ecosystem.
## Consequences
### Positive
- Prompt and progress output use compatible terminal tooling.
- Official examples provide implementation references.
### Negative
- Minegr becomes coupled to Dialoguer's prompt API.
- Some advanced prompts require optional features.
## Related
- [ADR 0001](0001-Interactive-prompt-crate.md)
- [Init command](../features/init-command.md)
- [Restart command](../features/restart-command.md)
- [Dialoguer examples](https://github.com/console-rs/dialoguer/tree/main/examples)
