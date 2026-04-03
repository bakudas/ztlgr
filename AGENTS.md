# AI Agent Guidelines for ztlgr

This document provides instructions for agentic coding systems working on the ztlgr Zettelkasten TUI application.

## Build & Test Commands

### Development Workflow

```bash
# Format code (run BEFORE commit)
cargo fmt --all

# Run clippy linting
cargo clippy --all-features -- -D warnings

# Run all unit tests
cargo test --lib

# Run single specific test
cargo test test_command_parser --lib -- --nocapture

# Build release binary
cargo build --release

# Development build
cargo run
```

### Makefile Shortcuts

```bash
make fmt                  # Format all code
make lint                 # Run clippy on all features
make test-unit            # Run unit tests (cargo test --lib)
make build                # Build release binary
make check                # Run fmt, lint, and test (pre-commit verification)
make clean                # Clean all build artifacts
```

### Pre-Commit Checklist

All commits MUST pass:

1. `cargo fmt --all` (no diffs allowed)
2. `cargo clippy --all-features -- -D warnings` (zero warnings)
3. `cargo test --lib` (all tests passing)
4. `cargo build` (release build succeeds)

Failure in any step means the commit cannot proceed.

---

## Code Style Guidelines

### General Principles

- **Rust Edition**: 2021 Edition with async support
- **Error Handling**: Use `Result<T>` return types exclusively; propagate errors with `?` operator
- **Naming**: Snake_case for functions/variables, PascalCase for types/traits
- **Documentation**: Public APIs require rustdoc comments with examples

### Imports

```rust
// Order: standard library, external crates, internal modules
use std::fmt;
use std::ops::Range;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;

use crate::config::Theme;
use crate::error::Result;
use super::GenericModal;
```

**Rules**:
- Group imports by category (std, external, internal)
- Use absolute paths for external crates and internal modules
- Use relative paths (`super::`, `crate::`) for internal hierarchy
- One import per line for clarity
- Remove unused imports (clippy enforces this)

### Type Definitions

```rust
// Public type aliases for ergonomics
pub type Result<T> = std::result::Result<T, ZtlgrError>;

// Prefer enums over booleans for state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Search,
    Command,
}

// Add traits as needed
#[derive(Debug, Clone)]
pub struct Note {
    id: String,
    content: String,
    // ...
}
```

### Error Handling

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZtlgrError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Note not found: {0}")]
    NotFound(String),
}

// Usage in functions
pub fn fetch_note(id: &str) -> Result<Note> {
    let note = db.get_note(id)
        .map_err(|_| ZtlgrError::NotFound(id.to_string()))?;
    Ok(note)
}

// Never use `.unwrap()` or `.expect()` in library code
// Always propagate errors or provide meaningful fallbacks
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_command_parser_help() {
        let cmd = CommandParser::parse("help");
        assert_eq!(cmd, Command::Help);
    }

    #[test]
    fn test_command_parser_with_args() {
        let cmd = CommandParser::parse("rename New Title");
        assert_eq!(cmd, Command::Rename("New Title".to_string()));
    }
    
    // Use tempfile for filesystem tests
    #[test]
    fn test_file_storage() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let vault = Vault::new(temp_dir.path())?;
        // test logic
        Ok(())
    }
}
```

**Rules**:
- All new code requires unit tests
- Place tests inline with `#[cfg(test)]` modules
- Use `Result<()>` for tests that may error
- Use `tempfile::TempDir` for filesystem tests
- Aim for 100% passing tests before commit

### Documentation

```rust
/// Creates a new note with the given title and content.
///
/// # Arguments
///
/// * `title` - The note title
/// * `content` - The note content (markdown or org format)
///
/// # Returns
///
/// Returns a `Note` with auto-generated ID and timestamp.
///
/// # Errors
///
/// Returns `ZtlgrError::InvalidNoteId` if title is empty.
///
/// # Examples
///
/// ```
/// let note = Note::new("My Note", "# Heading");
/// assert_eq!(note.title, "My Note");
/// ```
pub fn new(title: &str, content: &str) -> Result<Note> {
    if title.is_empty() {
        return Err(ZtlgrError::InvalidNoteId("title cannot be empty".into()));
    }
    // implementation
}
```

### Formatting

```rust
// 4-space indentation (cargo fmt enforces)
// Max line length: let rustfmt handle it (default ~100 chars)

// Complex match statements: break for readability
match command {
    Command::Rename(title) => {
        self.rename_note(&title)?;
        Ok(())
    }
    Command::Delete => {
        self.delete_note()?;
        Ok(())
    }
    _ => Err(ZtlgrError::Ui("Unknown command".into())),
}

// Trailing commas in multi-line structures
let config = Config {
    vault_path: "/home/user/.ztlgr",
    theme: "dark",
    auto_sync: true,
};
```

---

## AI Agent Team Structure

### Planning/Architecture Agent

**Responsibility**: Design and planning for new features.

**When to use**: 
- Breaking down complex features into phases
- Designing module structure and interfaces
- Planning database schema changes
- Identifying refactoring opportunities

**Output**: Detailed phase breakdown with test counts, module interfaces, database changes, and implementation steps.

**Example**: Designing Phase 5 (Link Detection) with:
- Module structure (link_parser.rs, link_validator.rs)
- Database schema for link tracking
- 30+ test cases for link formats
- Implementation phases (parser → validator → UI integration)

### Coder Agent

**Responsibility**: Implement features using TDD.

**When to use**:
- Implementing planned features
- Writing tests first, then implementation
- Following the established patterns in the codebase
- Creating new modules and functions

**Output**: Committed code with passing tests.

**Example**: Implementing a new command in Phase 4 (Command Mode):
1. Write 5-10 tests for the command
2. Implement the command variant in the Command enum
3. Add parser logic in CommandParser
4. Integrate into CommandExecutor
5. Verify all tests pass and commit

### Code Reviewer Agent

**Responsibility**: Quality assurance and style enforcement.

**When to use**:
- Before each commit
- To verify pre-commit checklist
- To ensure test coverage
- To check adherence to code style guidelines

**Output**: Approval or list of required changes.

**Checks**:
- `cargo fmt` compliance
- `cargo clippy` zero warnings
- `cargo test --lib` all passing
- `cargo build` succeeds
- Test coverage for new code
- Documentation for public APIs
- Error handling uses `Result<T>`
- No `.unwrap()` or `.expect()` in library code
- Import ordering and formatting

### UX/UI Lead Agent

**Responsibility**: User experience and interface design.

**When to use**:
- Designing new UI components
- Planning mode transitions
- Reviewing terminal output clarity
- Suggesting interaction improvements

**Output**: UI specifications and component designs.

**Focus areas**:
- Modal design and user flows
- Status bar messaging
- Key binding consistency
- Terminal width/height handling
- Color scheme and visibility

---

## Workflow Rules

### For All Agents

1. **TDD Mandatory**: Write tests before implementation
2. **Pre-commit**: Run full check sequence (`make check`)
3. **Commit Discipline**: One logical change per commit with conventional message
4. **100% Tests**: All tests must pass before pushing
5. **No Guessing**: Verify patterns in existing code before implementing new code

### Feature Implementation Flow

1. **Planning**: Planning/Architecture agent designs the feature
2. **Implementation**: Coder agent implements using TDD
3. **Review**: Code Reviewer agent verifies quality
4. **Commit**: Changes committed with descriptive message
5. **Validation**: Full test suite runs successfully

### Integration with Existing Code

- **Patterns to follow**: Modal system, Command parser, Storage organization
- **Module organization**: Keep related tests inline with `#[cfg(test)]`
- **Database**: SQLite with FTS5 via rusqlite
- **UI Framework**: Ratatui for widgets, Crossterm for events
- **Async**: Tokio runtime for async operations

---

## Quick Reference

### Test a Single Function

```bash
cargo test test_command_parser --lib -- --nocapture
```

### Format and Lint Before Commit

```bash
make fmt && make lint && make test-unit
```

### Verify Release Build

```bash
make build
```

### View Code Documentation

```bash
cargo doc --no-deps --open
```

### Check Project Status

```bash
cargo tree          # Dependency tree
cargo outdated      # Check for updates
```

---

## Repository Standards

- **Conventional Commits**: feat:, fix:, docs:, test:, refactor:, chore:
- **Test Coverage**: 100% of new code must have tests
- **Documentation**: All public APIs require rustdoc
- **No Direct Dependencies**: Avoid external crates not in Cargo.toml
- **File Organization**: Keep tests in same file as implementation with `#[cfg(test)]`

