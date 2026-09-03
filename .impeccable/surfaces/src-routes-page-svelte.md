---
version: 1
slug: 'src-routes-page-svelte'
primary_target: 'src/routes/+page.svelte'
related_targets: []
---

# Landing page

## Scope and mode

- Target: `src/routes/+page.svelte`
- Mode: Persuade
- One statically generated page for Cloudflare Pages.

## Audience, job, action, and proof

- Linux terminal users evaluating a pre-alpha Minecraft server manager.
- Explain the intended terminal-native server lifecycle without implying a release.
- Primary and only action: star the project on GitHub.
- Proof: a clearly labeled design preview based on documented `status` and `logs --follow` behavior, followed by the planned lifecycle stages.

## Chosen direction

- Approved comp: `.impeccable/mocks/option-a.png`.
- A broad green signal bleeds into a near-black hero, connecting an oversized wordmark to an overlapping terminal pane.
- The terminal crosses the hero boundary and resolves into a five-stage lifecycle rail.
- Memorable moment: the status signal travels once across the lifecycle rail and settles into the terminal cursor.

## Composition and implementation inventory

| Visible ingredient | Commitment                                                                    | Medium                                     |
| ------------------ | ----------------------------------------------------------------------------- | ------------------------------------------ |
| Hero statement     | `minegr` at monumental scale; white description immediately follows below     | Semantic heading and CSS                   |
| Green bleed        | Broad radial field covering the right half and feathering behind the wordmark | CSS radial gradients                       |
| Primary action     | One substantial `Star on GitHub` control below the hero copy                  | Semantic link and CSS                      |
| Terminal preview   | Large 16:9-ish pane offset right and overlapping hero/lifecycle boundary      | Semantic figure, HTML, and CSS             |
| Lifecycle rail     | Five connected stages across desktop; vertical, readable rows on mobile       | Semantic list and CSS                      |
| Motion             | One signal traversal into a blinking terminal cursor                          | CSS animation with reduced-motion fallback |
| Footer             | Small `minegr · Pre-alpha · Source on GitHub` line                            | Semantic footer and links                  |

## System extracted from the comp

- Component grammar: expansive page regions joined by one continuous rule; only the terminal uses a contained panel.
- Corners: terminal and CTA use 12px corners; page sections and lifecycle entries remain unboxed.
- Lines: one-pixel neutral rules; green only marks active lifecycle state and terminal data.
- Elevation: the terminal uses a deep, soft downward shadow without a colored halo.
- Type: heavy wide grotesk display, compact sans body, and tabular monospace only for terminal/data.
- Responsive rule: preserve hero priority, then stack CTA, terminal, and a vertical lifecycle rail without artificial viewport height.

## Constraints

- No literal Minecraft imagery, blocks, creatures, pixel fonts, or game screenshots.
- No invented release, customer, performance, or production-readiness claims.
- Repeatable content lives in typed TypeScript arrays and renders with Svelte `{#each}`.
- Keep the demo labeled as a design preview and every capability labeled as planned.
