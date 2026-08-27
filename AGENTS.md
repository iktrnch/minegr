# Repository Guidelines

## Project Structure

`minegr` is an early-stage Rust 2024 CLI/TUI for managing Minecraft servers on Linux.

- Keep `src/main.rs` focused on startup and command dispatch.
- Place reusable modules under `src/`.
- Store product and design documentation under `docs/`.
- Use the templates in `docs/_templates/` when creating documentation records.
- Keep Cargo output in the untracked `target/` directory.

## Development Commands

- `cargo check` — quick compilation feedback.
- `cargo run` — build and run the development binary.
- `cargo test` — run all tests.
- `cargo fmt --all -- --check` — verify formatting.
- `cargo clippy --all-targets --all-features -- -D warnings` — run lint checks.

Before completing a code change, run formatting, Clippy, and all relevant tests.

## Coding Conventions

Use standard `rustfmt` formatting and Rust naming conventions:

- `snake_case` for modules, functions, and files.
- `PascalCase` for types and traits.
- `SCREAMING_SNAKE_CASE` for constants.

Prefer small, focused modules and explicit error propagation over panics in operational paths.
Document public APIs and behaviour affecting server files, processes, backups, or updates.
Include /// Documentation comments which briefly explain what each function does.

## Feature and Refactor Workflow

For every feature or refactor request:

1. Inspect the relevant code and tests.
2. Read `docs/index.md`, relevant subsystem documentation, and applicable ADRs.
3. Invoke and follow the `grill-me` skill.
4. Ask only about material ambiguities not resolved by the request, code, tests, or documentation.
5. Do not begin implementation until the intended behaviour, scope, constraints, and acceptance criteria are clear.

Do not ask questions answered by the repository. State documented assumptions briefly. If sources conflict or appear outdated, explain the conflict and ask which direction to follow.

After requirements are clear, use the `tdd` skill where required below.

## Test-Driven Development

Invoke and follow the `tdd` skill for features and bug fixes with testable behaviour.

- Agree on the public testing seams before writing tests.
- Work in small red-green cycles, one behaviour at a time.
- Test observable behaviour through public interfaces.
- Keep tests deterministic and resilient to internal refactoring.
- Never weaken an assertion merely to make a test pass.

Place unit tests beside their code in `#[cfg(test)]` modules. Place cross-module and CLI-level tests under `tests/`. Use descriptive `snake_case` names such as `start_rejects_unknown_server`.

Cover successful behaviour and relevant failure cases from `docs/features/`. After implementation, run the relevant tests followed by the complete test suite and report the results.

If TDD is impractical for a change, state why before proceeding without it.

## Documentation

Documentation is stored under `docs/`, with `docs/index.md` as its entry point:

- `architecture/` — current system architecture.
- `features/` — implemented user-visible behaviour.
- `decisions/` — architectural decisions and their rationale.
- `proposals/` — unimplemented ideas, not current behaviour.

When making a significant change:

- Update documentation made inaccurate by the change.
- Update `docs/index.md` when documents are added or removed.
- Add useful links between related documents.
- Create an ADR for significant architectural decisions.
- Supersede historical ADRs instead of rewriting them.
- Move accepted proposal content into feature or architecture documentation after implementation.

Source code and tests are authoritative for implementation details. ADRs are authoritative for the rationale behind architectural decisions.

## Commits and Pull Requests

Use concise, imperative commit subjects such as `Add server status command`. Avoid vague subjects such as `wip`.

Pull requests should:

- Explain the problem and solution.
- List verification commands.
- Link relevant issues and design documents.
- Identify breaking CLI or configuration changes.
- Include terminal output or screenshots for TUI changes.
