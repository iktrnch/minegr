---
type: decision
status: accepted
created: 2026-08-24
supersedes: []
superseded_by: []
related_code: []
related_docs: []
---
## Context

`minegr init` requires interactive prompts for configuration values not supplied through CLI flags. These prompts include text input, platform and version selection, typed numeric input, validation, default values, help messages, and confirmations.

The prompt library must work alongside `clap`.

The main constraints are:
- Clear and consistent interactive user experience.
- Straightforward input validation with useful error messages.
- Support for text, selection, typed input, and confirmation prompts.
- Filtering or searching potentially long Minecraft version lists.
- Good documentation and maintainable APIs. 
- No requirement for asynchronous execution.
## Options considered
### Inquire

A Rust library for building interactive terminal prompts. It provides text, typed, selection, multiselection, confirmation, password, and editor prompts, alongside validation and autocompletion support.

**Advantages**

- Provides a polished interactive experience suitable for a setup wizard.
- Has clear and comprehensive documentation with practical examples.
- Supports typed prompts through `CustomType<T>`.
- Provides first-class input validation with user-facing error messages.
- Supports defaults, help messages, formatting, and autocompletion.
- Can filter or search long option lists, such as Minecraft versions.
- Its API is readable and already familiar from Ferium.

**Disadvantages**
- Adds a separate dependency alongside `clap`.
- Has its own rendering and styling system that must be configured consistently.
- Offers features that `minegr` may not initially require.
### Dialoguer

A Rust library providing common interactive terminal dialogs, including text input, selection, multiselection, confirmation, password input, fuzzy selection, and editor integration.

**Advantages**
- Supports all basic prompt types required by `minegr init`.
- Provides reusable themes for consistent prompt styling.
- Integrates naturally with the `console` and `indicatif` ecosystem.
- Supports optional fuzzy selection, completion, and input history.

**Disadvantages**
- Documentation is more reference-oriented and less approachable.
- Some useful functionality requires optional feature flags.
- Validation and prompt configuration are less expressive for a multi-step setup wizard.
- Its API and resulting interaction style are less familiar to the project’s developer.
## Decision

Use `inquire` for interactive terminal prompts.

Continue using `clap` for command and flag parsing. Values supplied through `clap` flags will bypass their corresponding `inquire` prompts.
## Rationale

Both libraries provide the basic prompt types required by `minegr` - `inquire` was chosen because its API, documentation, validation model, and default presentation are better suited to the planned `minegr init` setup wizard.

Its support for typed input, inline validation, help messages, defaults, and searchable selections reduces the amount of custom interaction code required. Its interaction style is also familiar from [Ferium](https://github.com/gorilla-devs/ferium) and matches the desired user experience.

The choice is based primarily on maintainability and developer experience rather than a capability missing entirely from `dialoguer`.
## Consequences
### Positive
- Interactive setup can provide clear defaults, help text, and validation errors.
- Minecraft versions can be presented through searchable selection prompts.
- Prompt code should remain concise and readable.
- The interaction style will be consistent with the intended CLI experience. 
- Documentation should make future prompt development easier.
### Negative
- `inquire` becomes an additional project dependency.
- Prompt behaviour and styling will be coupled to `inquire`.
- Replacing it later would require changing the interactive input layer.
- Non-interactive execution must be implemented separately through `clap`; `inquire` does not replace argument parsing.   
## Related
- `minegr init` command specification.
- `clap` command-line argument parsing.
- `minegr.toml` configuration format.
- Non-interactive initialization behaviour.