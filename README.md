# ⚡ DevOS — Developer Workspace

A Rust desktop app that gives developers a local AI-powered workspace — like a mini VS Code + Terminal + Git + AI assistant, running 100% locally with $0 external services.

![Rust](https://img.shields.io/badge/Rust-1.93+-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Status](https://img.shields.io/badge/status-MVP-yellow)

## Features

- **📁 File Explorer** — Browse project files with a collapsible tree sidebar
- **📝 Code Editor** — Tabbed editor with syntax highlighting, Ctrl+S save, unsaved indicators
- **💻 Terminal Runner** — Run any shell command, view live output, stop processes, command history
- **⎇ Git Panel** — View branch, changed files, stage/unstage, commit — all via `libgit2`
- **🤖 AI Chat** — Talk to a local AI (Ollama) with file context, get code explanations and patches
- **🩹 Patch Apply** — Preview diffs, apply/reject patches safely with automatic backups

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| GUI | egui via eframe |
| Async | tokio |
| Git | git2 (libgit2) |
| AI | Ollama (local HTTP API via reqwest) |
| File Dialog | rfd |
| Diff/Patch | diffy |

## Prerequisites

- **Rust** (1.70+): Install via [rustup](https://rustup.rs)
- **C Compiler**: Visual Studio Build Tools (Windows) or `gcc` (Linux/Mac) — required by `git2`
- **Ollama** (optional): Install from [ollama.ai](https://ollama.ai) for AI features

## Quick Start

```bash
# Clone
git clone https://github.com/Muyideen-js/DevOs.git
cd DevOs

# Build and run
cargo run

# Run tests
cargo test
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── app.rs               # eframe::App — wires all panels together
├── state.rs             # AppState with all sub-states
├── models.rs            # Shared data types
├── lib.rs               # Library crate for tests
├── core/                # Business logic (no UI)
│   ├── file_manager.rs  # File I/O, dir tree, backups
│   ├── terminal.rs      # Command execution + streaming
│   ├── git.rs           # git2 wrappers
│   ├── chat.rs          # Ollama HTTP integration
│   ├── patch.rs         # Diff parsing + apply
│   └── project.rs       # Recent projects persistence
└── ui/                  # egui rendering
    ├── top_bar.rs       # Project name, open folder, settings
    ├── file_explorer.rs # Left sidebar tree
    ├── editor.rs        # Tabbed code editor
    ├── terminal.rs      # Bottom terminal panel
    ├── git_panel.rs     # Right panel — Git
    ├── chat_panel.rs    # Right panel — AI Chat
    ├── patch_view.rs    # Diff preview + apply/reject
    └── settings.rs      # Settings modal
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+S` | Save current file |
| `Ctrl+F` | Toggle search in file |
| `Ctrl+`` ` | Toggle terminal panel |

## Roadmap

- [x] Project open + file tree
- [x] Tabbed editor + save
- [x] Terminal runner + stop + history
- [x] Git status + stage/unstage + commit
- [x] AI chat + context selection
- [x] Ollama integration
- [x] Patch preview + apply + backup
- [x] Settings + recent projects
- [ ] File watcher (auto-refresh tree)
- [ ] Tree-sitter syntax highlighting
- [ ] Multi-cursor editing
- [ ] LSP autocompletion

## License

MIT
