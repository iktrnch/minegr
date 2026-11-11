<div align="center">

  <img src="./assets/minegr.svg" width="620" alt="minegr logo">

  <p><strong>Run Minecraft servers from the terminal, not a browser panel.</strong></p>

  [![Rust 2024](https://img.shields.io/badge/Rust-2024-000000?style=flat-square&logo=rust&logoColor=white)](./Cargo.toml)
  [![Platform: Linux](https://img.shields.io/badge/platform-Linux-FCC624?style=flat-square&logo=linux&logoColor=black)](./docs/architecture/filesystem-layout.md)
  [![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-F59E0B?style=flat-square)](#project-status)

  [Overview](#overview) · [Quick start](#quick-start) · [Commands](#commands) · [Configuration](#configuration) · [Development](#development)

</div>

## Overview

> [!Warning]
> `minegr` is in **very early development**. It is not currently ready for production use, and features, configuration formats, commands, and architecture may change significantly.

**minegr** *(pronounced “mine-ger”, IPA: `/ˈmaɪn.dʒər/`)* is a Minecraft server manager written in Rust and designed around a fast CLI and TUI.

The goal is to make creating and operating Minecraft servers on Linux simple without requiring a browser-based management panel.

## Expected features

* Create and import Minecraft servers
* Install and manage server software such as Vanilla, Paper, and Fabric
* Start, stop, restart, and monitor servers
* Interactive server console
* View and follow server logs
* Create and restore backups
* Check for and install server updates
* Manage common server configuration
* Monitor CPU, memory, uptime, and server status
* Create reproducible server environments for quick deployment and sharing
* CLI commands suitable for scripting and automation
* Interactive terminal UI for day-to-day administration
* Plugin and mod management later in development

## Who is it for?

`minegr` is primarily intended for people who run Minecraft servers on Linux and are comfortable working in a terminal, including:

* Self-hosters and homelab users
* Minecraft server administrators
* Developers
* VPS and dedicated-server users
* Anyone who wants a lightweight alternative to a full web management panel

The project is **not initially aimed at replacing browser-based hosting panels** for non-technical users or large commercial hosting providers.

## Status

🚧 **Pre-alpha / early development**

The project is currently being designed and implemented. Expect incomplete functionality and breaking changes.
