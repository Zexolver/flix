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

## ✅ Completed (v0.1.2 Infrastructure)
- [x] **ARM64 Linker Support**: Added `.cargo/config.toml` to force `lld` linker, fixing relocation errors on Zinuxbook/AArch64.
- [x] **Global System Integration**: Established `/usr/local/flix/bin` and `/usr/local/flix/etc` as standard global paths.
- [x] **Dynamic Shell Completion**: Integrated `clap-complete` with a custom Bash/Zsh dynamic wrapper that reads installed packages directly from the TOML for `update` and `remove` commands.
- [x] **Shell-Init Logic**: Automates PATH updates and completion sourcing for `.bashrc`, `.zshrc`, and `.profile`.
- [x] **Modularized Engine**: Cleaned up the `engine.rs` monolith into separate modules.

## 🚧 Immediate Next Steps (v0.1.2)
- [ ] **GitHub Release Scraper**: Implement logic in `installer.rs` to fetch pre-built binaries from GitHub Releases to avoid long compile times when a binary is available.
- [ ] **Flix Doctor**: Add a `doctor` command to verify PATH health, directory permissions, and linker availability.
- [ ] **Concurrency**: Implement parallel processing for the `flix update` command to check multiple git repos at once.
- [ ] **Lockfile Implementation**: Add a `flix.lock` to ensure deterministic builds across different machines.

## 🔮 Future Backlog
- [ ] Cross-platform support (Windows/macOS optimizations).
- [ ] Build the `flix-ui` crate for an interactive terminal dashboard.
- [ ] Multi-threading for parallel package updates.
