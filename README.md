<div align="center">

# 🧹 Cargo Thin (カーゴ・シン)
### *Intelligent Rust Target Purger & High-Performance Ratatui TUI*

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange?style=flat-square)](https://crates.io)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux--x86__64-informational?style=flat-square)](https://github.com/Praveensenpai/cargo-thin)
[![Built with Ratatui](https://img.shields.io/badge/TUI-Ratatui%200.30-purple?style=flat-square)](https://ratatui.rs)

**Preserve your compiled executable binaries while reclaiming tens of gigabytes of intermediate build bloat (`deps/`, `incremental/`, `build/`, `.fingerprint/`).**

---

</div>

## 💡 Why Cargo Thin?

Standard `cargo clean` is a **dumb cleaner**: it blindly obliterates the entire `target/` directory, wiping out your freshly compiled binaries along with intermediate cache files. The next time you need to run your CLI tools, you are forced to recompile everything from scratch.

**Cargo Thin** is an **intelligent cleaner**:
- 🔍 **Manifest-Aware Binary Discovery**: Inspects `Cargo.toml` definitions (`[[bin]]`, `src/main.rs`, `src/bin/*.rs`) to locate real executables.
- 🛡️ **ELF Header Validation**: Inspects executable headers (`\x7fELF`) to distinguish true binaries from shared libraries (`.so`, `.rlib`) or corrupted files.
- 🔒 **Atomic Safe-Staging**: Stashes your binaries in a lock directory before purging intermediate caches, then restores them to their canonical `./target/release/` or `./target/debug/` paths with executable permissions (`0755`).
- ⚡ **Zero Broken Paths**: Any launcher, shell alias, or script targeting `./target/release/<binary>` continues to work uninterrupted.
- 📊 **Beautiful Ratatui TUI**: Real-time project sorting, disk bloat gauges, and tree inspector.

---

## 🖥️ TUI Interface

```
┌─ Cargo Thin (カーゴ・シン) ─────────────────────────────── v0.1.0 ─┐
│ Projects: 9  │ Target Bloat: 32.59 GB │ Reclaimable: 32.15 GB (99%) │
├──────────────────────────────────────┬──────────────────────────────┤
│ [ ] deeperseeker     13.05 GB   97%  │ INSPECTOR: deeperseeker      │
│ [x] nagare            5.78 GB  100%  │ Location: ~/Projects/nagare  │
│ [x] kotonoha          3.49 GB  100%  │                              │
│ [x] seedr-relay       2.98 GB  100%  │ Target Breakdown:            │
│ [x] karakuri          2.21 GB  100%  │   deps/          4.20 GB ──┐ │
│ [x] omarchy-dotfiles  1.91 GB  100%  │   incremental/   1.10 GB   │ │
│ [x] seedr-dl          1.60 GB  100%  │   build/       480.00 MB   │P│
│ [x] aur-builder     673.59 MB   99%  │   .fingerprint/   2.40 MB   │U│
│                                      │   -----------------------   │R│
│                                      │   Reclaimable:   5.77 GB ──┘G│
│                                      │   Preserved:     6.70 MB ──┐E│
│                                      │    ✓ [ELF] release/nagare   │K│
│                                      │                             │E│
│                                      │ Savings: [██████████░] 100% │E│
│                                      │                             │P│
├──────────────────────────────────────┴──────────────────────────────┤
│ [Space] Toggle  [a] All  [s] Sort  [p] Policy  [c] Clean  [q] Quit  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## ⚡ Quick Start

### Installation

```bash
cargo install --path /home/paisen/Projects/cargo-thin
```

### Launch Interactive TUI

```bash
# Launch interactive TUI for current workspace
cargo thin

# Or inspect a specific directory
cargo thin --path ~/Projects
```

### Non-Interactive / Scriptable CLI

```bash
# Dry-run audit (prints sorted table without touching files)
cargo thin --path ~/Projects --dry-run

# Clean all projects non-interactively
cargo thin --path ~/Projects --all --yes

# JSON export for CI / disk monitoring
cargo thin --path ~/Projects --json
```

---

## ⌨️ Keyboard Shortcuts

| Key | Action |
|:---|:---|
| <kbd>j</kbd> / <kbd>↓</kbd> | Move cursor down |
| <kbd>k</kbd> / <kbd>↑</kbd> | Move cursor up |
| <kbd>Space</kbd> | Toggle project selection `[✓]` |
| <kbd>a</kbd> | Select All / Deselect All |
| <kbd>s</kbd> | Cycle sort order (*Target Size Desc* → *Reclaimable Desc* → *Name Asc*) |
| <kbd>p</kbd> | Toggle preservation policy (*Prefer Release* vs *Keep Both*) |
| <kbd>c</kbd> | Open confirmation modal & clean selected |
| <kbd>?</kbd> | Open help dialog |
| <kbd>q</kbd> / <kbd>Ctrl+C</kbd> | Quit application |

---

## 🛡️ Safety Guarantees

> [!IMPORTANT]
> `cargo-thin` never deletes source code, `.git`, `Cargo.toml`, or non-target directories.

> [!TIP]
> Binaries are validated using ELF header magic (`\x7fELF`) and file execution mode (`0755`) before any purge. If a binary is unverified, `cargo-thin` halts cleanup for that project rather than risking data loss.

---

## 📜 License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
