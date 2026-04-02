# ztlgr - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure
- SQLite database schema with FTS5 support
- Note types (daily, fleeting, literature, permanent, reference, index)
- Zettelkasten ID system (Luhmann-style)
- Configuration system with TOML
- Theme system (Dracula, Gruvbox, Nord, Solarized, custom)
- Basic TUI with Ratatui
- Note listing sidebar
- Note editor
- Preview pane
- Status bar
- Vim-style keybindings (normal, insert, search modes)
- YAML frontmatter support
- Link detection infrastructure
- Tag system infrastructure

### Security
- Soft delete for notes (data retention)
- Vault-level isolation (each vault is a separate SQLite file)

## [0.1.0] - 2024-01-XX

### Added
- Project initialization
- Basic architecture with multi-agent system
- Database layer with SQLite
- Note CRUD operations
- Full-text search setup
- Theme support
- Configuration management