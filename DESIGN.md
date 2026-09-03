---
name: Minegr
description: A dark operational design system shaped by one live green signal.
colors:
  signal: 'oklch(69.6% 0.17 162.48)'
  signal-bright: 'oklch(76.5% 0.177 163.223)'
  signal-ink: 'oklch(26.2% 0.051 172.552)'
  field: '#030806'
  terminal: '#070a08'
  terminal-chrome: '#090d0b'
  text: 'oklch(98.5% 0 0)'
  text-muted: 'oklch(70.5% 0.015 286.067)'
  rule: 'oklch(37% 0.013 285.805)'
typography:
  display:
    fontFamily: 'Archivo Variable, sans-serif'
    fontSize: 'clamp(5.25rem, 15vw, 13rem)'
    fontWeight: 900
    lineHeight: 0.82
    letterSpacing: '-0.055em'
  headline:
    fontFamily: 'Archivo Variable, sans-serif'
    fontSize: 'clamp(2.15rem, 4.8vw, 4.6rem)'
    fontWeight: 400
    lineHeight: 1.08
    letterSpacing: '-0.035em'
  body:
    fontFamily: 'Archivo Variable, sans-serif'
    fontSize: '0.875rem'
    fontWeight: 400
    lineHeight: 1.5rem
  mono:
    fontFamily: 'JetBrains Mono Variable, monospace'
    fontSize: '0.75rem'
    fontWeight: 400
    lineHeight: 1.7
  label-compact:
    fontFamily: 'JetBrains Mono Variable, monospace'
    fontSize: '0.65rem'
    fontWeight: 400
    letterSpacing: '0.12em'
  label-preview:
    fontFamily: 'JetBrains Mono Variable, monospace'
    fontSize: '0.68rem'
    fontWeight: 400
    letterSpacing: '0.08em'
rounded:
  control: '12px'
  panel: '14px'
  circle: '9999px'
spacing:
  tight: '8px'
  control: '12px'
  content: '20px'
  panel: '28px'
  gutter: '56px'
components:
  button-primary:
    backgroundColor: '{colors.signal}'
    textColor: '{colors.signal-ink}'
    rounded: '{rounded.control}'
    padding: '14px 28px'
    height: '52px'
  button-primary-hover:
    backgroundColor: '{colors.signal-bright}'
    textColor: '{colors.signal-ink}'
  terminal-panel:
    backgroundColor: '{colors.terminal}'
    textColor: '{colors.text}'
    rounded: '{rounded.panel}'
    padding: '28px'
  stage-marker:
    backgroundColor: '{colors.field}'
    textColor: '{colors.signal-bright}'
    rounded: '{rounded.circle}'
    size: '40px'
---

# Design System: Minegr

## Overview

**Creative North Star: "The Live Server Signal"**

Minegr treats the interface as one operational field: near-black space is interrupted by a broad green signal, precise rules, and the occasional contained terminal surface. Scale and overlap make the work memorable; restrained controls keep it credible.

The system is technical without dressing every element as a terminal. Monospace belongs only to commands, state, and measurements. Literal Minecraft imagery, pixel styling, game screenshots, and block motifs do not belong in the visual identity.

**Key Characteristics:**

- Monumental wide display type against quiet open space.
- One committed emerald signal over charcoal-black fields.
- Exact rules and connected sequences instead of generic card grids.
- Matte surfaces with one selectively elevated operational pane.
- A single authored motion event with reduced-motion support.

## Colors

The palette is a restrained primary-plus-neutral system: emerald carries state and emphasis while near-black surfaces provide the operating field.

### Primary

- **Live Signal:** Drives product identity, the primary action, active states, and connected process lines.
- **Signal Bright:** Handles hover and high-contrast status details.
- **Signal Ink:** Keeps dark text legible on bright green controls.

### Neutral

- **Night Field:** Owns the page background and open negative space.
- **Terminal Well:** Holds terminal content beneath its darker chrome.
- **Paper White:** Carries the product descriptor and primary reading text.
- **Muted Zinc:** Supports secondary copy, labels, and terminal prompts.
- **Rule Zinc:** Connects lifecycle stages and separates surfaces.

**The Signal Field Rule.** Green may own a large continuous region or one active state; do not scatter small green decorations across unrelated elements.

## Typography

**Display Font:** Archivo Variable (with sans-serif fallback)  
**Body Font:** Archivo Variable (with sans-serif fallback)  
**Label/Mono Font:** JetBrains Mono Variable (with monospace fallback)

**Character:** Archivo supplies a wide, heavy wordmark and a plainspoken reading voice without becoming a retro-game reference. JetBrains Mono is functional evidence, never a general technical costume.

### Hierarchy

- **Display:** Very heavy, viewport-responsive, and compact without colliding with following copy; reserved for product identity.
- **Headline:** Regular weight with compact spacing; pairs directly with the display wordmark.
- **Title:** Semibold at 1.5rem; names lifecycle stages.
- **Body:** Regular at 0.875rem with 1.5rem line height; supports short factual explanations.
- **Label:** Uppercase mono at 0.65–0.75rem with visible tracking; restricted to real state and preview metadata.

**The Evidence Mono Rule.** Use monospace only for commands, measurements, and status metadata.

## Layout

Content sits in a fluid container capped at 96rem with compact 20px mobile gutters, 32px tablet gutters, and 56px desktop gutters. Desktop compositions use asymmetric overlap: open editorial copy can hand off to a contained operational surface without wrapping both in a shared card.

Connected sequences may use five equal columns at large sizes. Below 64rem, they become full-width vertical rows and the overlap resolves into normal document flow. Preserve the dominant reading order at every breakpoint instead of shrinking the desktop composition wholesale.

## Elevation & Depth

The system is flat by default. Depth comes from field contrast, overlap, and one ambient terminal shadow (`0 28px 80px rgba(0,0,0,0.48)`). The primary action may gain a softer downward hover shadow, but colored halos are not a depth language.

**The One Lifted Surface Rule.** Elevate the active operational surface; leave surrounding page regions on the field.

## Shapes

Controls use confident 12px corners and operational panels use 14px corners. Status points and sequence markers are circles. Page regions, lifecycle entries, and text groups remain unboxed and rely on rules and spacing.

## Components

### Buttons

- **Shape:** A substantial rectangular control with gently curved corners (12px).
- **Primary:** Live Signal background, Signal Ink text, 28px horizontal padding, and 52px minimum height.
- **Hover / Focus:** Brighten one palette step and lift by 2px on hover; use an offset Signal Bright focus outline.

### Terminal Panel

- **Shape:** A single matte operational pane with 14px corners.
- **Surface:** Dark chrome over the Terminal Well, divided by a one-pixel zinc rule.
- **Content:** Render commands and measurements as real text from structured data; highlight only the command name and healthy state.
- **Motion:** A step-timed cursor blink is allowed and must stop for reduced-motion preferences.

### Lifecycle Rail

- **Structure:** An ordered sequence connected by one exact rule; markers carry meaningful stage numbers.
- **Responsive behavior:** Five columns on wide screens and readable full-width rows on compact screens.
- **State:** Planned status is muted; hover may fill a marker without changing the content hierarchy.

## Do's and Don'ts

### Do:

- **Do** use broad green fields to connect content regions.
- **Do** create hierarchy with scale, overlap, and negative space.
- **Do** keep planned functionality visibly labeled during pre-alpha.
- **Do** keep repeated content in typed data structures rather than duplicating markup.

### Don't:

- **Don't** use literal Minecraft imagery, pixel fonts, creatures, blocks, or game screenshots as brand language.
- **Don't** turn technical content into neon, glass, or glow-heavy chrome.
- **Don't** use same-size icon cards as the page structure.
- **Don't** use monospace outside code, data, and real operational metadata.
