# 🚀 Flix Package Manager - AI Context & Roadmap

## 🧠 Project Overview
Flix is a blazingly fast package manager written in Rust. It allows users to install binaries directly from Git repositories or GitHub Releases. It supports custom installation paths, tag filtering, and shell integration. 

## 📂 Architecture
The project is divided into three crates:
* **`flix-cli`**: Handles the command-line interface using `clap`. Parses arguments and passes them to the core engine.
* **`flix-core`**: The brains. Modularized into:
  * `config.rs`: Manages `config.toml` state.
  * `flags.rs`: Contains `SharedArgs` (global flags like quiet, force, tags).
  * `engine/`: Domain-specific logic (`builder.rs`, `git_manager.rs`, `installer.rs`, `registry.rs`, `system.rs`).
* **`flix-ui`**: Reserved for a future Terminal User Interface (TUI).

## ✅ Completed (v0.1.1 Foundation)
- [x] Implement `SharedArgs` for global flags (`-q`, `-f`, `-t`, `-p`).
- [x] Support checking out specific Git tags/commits using `-V` (`--git-ref`).
- [x] Fix `clap` naming conflicts between global version and git-ref version.
- [x] Modularize `engine.rs` into a clean directory structure to prevent AI hallucinations.
- [x] Add quiet mode (`-q`) to suppress Cargo build noise.

## 🚧 Immediate Next Steps
- [ ] **GitHub Release Scraper**: Update `installer.rs` to support downloading pre-built binaries from GitHub Releases instead of always compiling from source.
- [ ] **Update Command**: Polish the `flix update` logic to handle pulling new commits and rebuilding.
- [ ] **Remove Command**: Polish the auto-confirm (`-y`) logic for `flix remove`.

## 🔮 Future Backlog
- [ ] Cross-platform support (Windows/macOS optimizations).
- [ ] Build the `flix-ui` crate for an interactive terminal dashboard.
- [ ] Multi-threading for parallel package updates.
