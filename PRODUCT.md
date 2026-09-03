# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Minegr is for Linux users who operate Minecraft servers from a terminal: self-hosters, homelab users, server administrators, developers, and VPS or dedicated-server users.

## Product Purpose

Minegr is a Minecraft server manager written in Rust and designed around a fast CLI and TUI. It aims to make creating and operating a Linux-hosted server straightforward without requiring a browser-based management panel.

The project is currently pre-alpha and in active design and implementation. The landing page should help visitors understand the intended workflow and follow development on GitHub without implying that Minegr is production-ready.

## Positioning

Minegr keeps server creation, lifecycle control, status, logs, console access, and backups in a terminal-native workflow rather than moving administration into a web panel.

## Operating Context

Users work in Linux terminals and manage servers through commands such as `minegr init`, `minegr start`, `minegr status`, `minegr logs --follow`, `minegr console`, and `minegr backup`.

## Capabilities and Constraints

- The documented near-term workflow covers configuration, start and stop, status and process metrics, live logs, an interactive console, and backups.
- These workflows are planned and currently documented as unimplemented.
- The site is one statically generated SvelteKit page deployed to Cloudflare Pages.
- Repeatable marketing content and demonstration lines must live in typed TypeScript arrays and render through Svelte `{#each}` blocks so the page can change easily during pre-alpha development.
- The page has one primary action: star the project on GitHub.

## Brand Commitments

- Product name: `minegr`.
- Hero line: `minegr — a Minecraft server manager for your terminal`.
- Dark visual world with green derived from Tailwind CSS colors.
- No literal Minecraft imagery or overt game styling. Restrained grid or block-derived structure and familiar green hues are allowed.
- Voice is concise, technical, factual, and explicit about pre-alpha status.
- New landing-page surfaces use a comp-led execution path: approve a high-fidelity composition before implementation.

## Evidence on Hand

- Product and architecture documentation: `../minegr/docs/`.
- Product overview: `../minegr/README.md`.
- Repository: `https://github.com/iktrnch/minegr`.
- No release, production-readiness claim, customer proof, benchmark, or testimonial is available and none may be invented.
- The terminal demonstration is a labeled design preview of documented planned behavior.

## Product Principles

- Make the terminal-native workflow understandable in one viewport.
- Demonstrate documented behavior instead of making broad claims.
- Mark planned and pre-alpha work honestly.
- Keep page content simple to update as the project changes.
- Prefer one clear action over competing conversion paths.
