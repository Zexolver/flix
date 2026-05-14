# Flix — AI Context & Developer Guide

Welcome, AI collaborator. This document outlines the architecture, design philosophies, and current state of **Flix** to ensure code generation matches the project's exact mental model.

## 🚀 Project Overview
Flix is a fast, lightweight package/binary manager written in Rust. It allows users to install tools either via pre-built GitHub releases (preferred) or by automatically compiling from source.

- **Current Version:** v0.1.3
- **Target OS/Arch:** Linux / MacOS (Primary dev machine: `aarch64-linux` / Linux ARM64).
- **Core Principles:** Zero unnecessary dependencies, clean error propagation, fast execution, human-readable config.

---

## 📂 Project Structure
Flix is structured as a Cargo workspace with three primary crates:
- `flix-cli/`: The command-line entry point (`main.rs`, parsing flags via `clap`).
- `flix-ui/`: Optional visual/TUI interface components.
- `flix-core/`: The entire execution engine.
  - `src/config.rs`: Manages `~/.config/flix/config.toml` using `toml_edit`.
  - `src/engine/installer.rs`: The core install loop (checks release -> falls back to source).
  - `src/engine/platform.rs`: Detects system OS/Arch and handles fuzzy match logic (`aarch64` vs `arm64`).
  - `src/engine/git_manager.rs`: Clones repositories for source builds.
  - `src/engine/builder.rs`: Automatically detects build systems (Cargo, Makefile, Go) and compiles.
  - `src/engine/system.rs`: Low-level file manipulation, directory verification, and `sudo` handling.

---

## 💡 Important Logic & "Gotchas"

### 1. The GitHub Scraping Secret (`expanded_assets`)
GitHub hides its release asset tables behind a client-side JavaScript loading state, returning an empty table to basic HTTP clients. 
To bypass this without requiring a GitHub API token, `installer.rs` uses a custom scraping trick:
- It targets the hidden endpoint: `{repo_url}/releases/expanded_assets/{tag}`
- This returns a raw HTML snippet containing *only* the asset download links, which `scan_html_for_link` parses via delimiter splitting.

### 2. Fuzzy Architecture Matching
Filenames across GitHub use inconsistent naming conventions. `platform.rs` generates a vector of search terms to accommodate this:
- If host is `aarch64`, terms searched include both `aarch64` and `arm64`.
- If host is `x86_64`, terms searched include both `x86_64` and `amd64`.

### 3. Binary Unpacking Heuristics
When downloading a `.tar.gz` or `.tgz` release, the installer iterates through the archive entries and looks for a file containing or matching the `package_name` string to extract as the executable binary.

---

## 🛠 Coding Conventions
- **Early Returns:** Prefer `if let` or `match` with early returns instead of deep nested blocks.
- **Error Handling:** Use `.ok()?` or explicit logs over raw `.unwrap()`. Avoid panicking in core execution.
- **Sudo Execution:** Moving binaries to `/usr/local/flix/bin/` requires elevated permissions, managed carefully inside `system::copy_with_sudo`.

---

## 🎯 Upcoming Roadmap: v0.1.4 (Refactoring & Simplification)
The primary objective of the next version is **not** adding features, but architectural cleanup:
1. **File Splitting:** `installer.rs` currently handles URL parsing, HTML scraping, downloading, and archive extraction. These need to be broken out into single-responsibility sub-modules (e.g., `scraper.rs`, `downloader.rs`, `extractor.rs`).
2. **Comprehensive Comments:** Add structured documentation (`///`) to every public function explaining inputs, outputs, and edge cases.
3. **Data Type Hardening:** Pass strongly-typed abstractions where possible instead of loose strings (e.g., proper URL types or Tag wrappers).
