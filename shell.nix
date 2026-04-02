{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt
    
    # Build dependencies
    pkg-config
    openssl
    
    # SQLite
    sqlite
    
    # For TUI
    ncurses
    
    # Development tools
    cargo-watch
    cargo-edit
  ];
  
  # Environment variables
  RUST_BACKTRACE = "1";
  RUST_LOG = "info";
  
  # SQLite paths
  SQLITE_LIB_DIR = "${pkgs.sqlite.out}/lib";
  SQLITE_INCLUDE_DIR = "${pkgs.sqlite.dev}/include";
  
  # OpenSSL paths
  OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
  OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
  
  shellHook = ''
    # Set up PATH
    export PATH=$PATH:~/.cargo/bin
    
    # Aliases
    alias run='cargo run'
    alias test='cargo test --all-features'
    alias lint='cargo clippy --all-features -- -D warnings'
    alias fmt='cargo fmt --all'
    alias watch='cargo watch -x run'
    
    echo "🦀 ztlgr development environment (shell.nix)"
    echo ""
    echo "Available commands:"
    echo "  run         - Run the application"
    echo "  test        - Run tests"
    echo "  lint        - Run clippy"
    echo "  fmt         - Format code"
    echo "  watch       - Watch and rebuild"
    echo ""
    echo "Note: For better caching, consider using flake.nix with direnv"
  '';
}