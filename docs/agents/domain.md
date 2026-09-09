# Domain Docs

How engineering skills should consume Minegr's domain documentation.

## Source of truth

This is a single-context repository. The `docs/` tree is the behavioural source of truth for planned and documented Minegr behaviour.

Before exploring or changing a subsystem:

1. Read [`docs/index.md`](../index.md) to locate the relevant records.
2. Read the applicable documents in [`docs/architecture/`](../architecture/).
3. Read related accepted records in [`docs/decisions/`](../decisions/).
4. Read the relevant user-visible contracts in [`docs/features/`](../features/).
5. Use [`docs/glossary.md`](../glossary.md) for established terminology.

`docs/proposals/` contains unimplemented ideas and is not current behavioural authority.

## Repository layout

```text
docs/
├── architecture/  current system design
├── decisions/     accepted architectural rationale
├── features/      user-visible behavioural contracts
├── proposals/     unimplemented ideas
└── glossary.md    established terminology
```

Do not create or require `CONTEXT.md`, `CONTEXT-MAP.md`, or `docs/adr/` for this repository. Use the existing `docs/decisions/` and `docs/architecture/` structure.

## Conflicts

When code, a feature contract, architecture record, or accepted decision conflicts with another source, identify the conflict explicitly. Do not silently override an accepted decision; update or supersede it through the repository's existing documentation conventions when the user authorizes that change.
