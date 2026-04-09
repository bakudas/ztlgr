# Contributing to ztlgr

Thank you for your interest in contributing to ztlgr!

## Development Setup

### Using Nix (Recommended)

```bash
# Enter development shell
nix-shell
# or with flakes
nix develop

# Or use direnv
direnv allow
```

### Without Nix

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies (Ubuntu/Debian)
sudo apt-get install pkg-config libssl-dev libsqlite3-dev

# Install dependencies (macOS)
brew install openssl sqlite

# Build
cargo build

# Run tests
cargo test

# Run
cargo run
```

## Development Commands

```bash
# Run in development mode
make dev

# Build release
make build

# Run tests
make test

# Lint code
make lint

# Format code
make fmt

# Generate docs
make doc

# Watch and rebuild
make watch
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library root
├── error.rs             # Error types
├── config/              # Configuration and themes
├── db/                  # Database layer (SQLite + FTS5)
├── note/                # Note types and Zettelkasten
├── link/                # Link parsing, validation, fuzzy matching
├── graph/               # Knowledge graph types and layout
├── storage/             # File storage (Markdown/Org), sync, import
├── ui/                  # TUI interface (ratatui)
└── setup.rs             # Setup wizard
```

## Architecture

ztlgr uses a hybrid architecture:

- **Files as Source of Truth**: Notes stored as .md/.org files
- **SQLite as Index**: Fast search and relationships
- **File Watcher**: Auto-sync on changes

## Code Style

- Follow Rust best practices
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add tests for new functionality
- Document public APIs with rustdoc

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation
- `test:` Tests
- `refactor:` Refactoring
- `chore:` Maintenance

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_note_creation

# Run with coverage
cargo tarpaulin
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and lints
5. Submit PR

## Questions?

Open an issue or join discussions!