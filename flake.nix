{
  description = "ztlgr - A terminal-based note-taking app with Zettelkasten methodology";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };
        
        buildInputs = with pkgs; [
          # Build dependencies
          pkg-config
          openssl
          
          # SQLite
          sqlite
          
          # For TUI
          ncurses
        ];
        
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          shellHook = ''
            # Set up environment
            export PATH=$PATH:~/.cargo/bin
            
            # Rust environment
            export RUST_BACKTRACE=1
            export RUST_LOG=info
            
            # SQLite
            export SQLITE_LIB_DIR=${pkgs.sqlite.out}/lib
            export SQLITE_INCLUDE_DIR=${pkgs.sqlite.dev}/include
            
            # OpenSSL
            export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
            export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
            
            # Aliases
            alias run='cargo run'
            alias test='cargo test --all-features'
            alias lint='cargo clippy --all-features -- -D warnings'
            alias fmt='cargo fmt --all'
            
            echo "🦀 ztlgr development environment"
            echo ""
            echo "Available commands:"
            echo "  run         - Run the application"
            echo "  test        - Run tests"
            echo "  lint        - Run clippy"
            echo "  fmt         - Format code"
            echo ""
            echo "Use 'direnv allow' to activate this environment."
          '';
        };
        
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ztlgr";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          
          inherit buildInputs nativeBuildInputs;
        };
        
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      }
    );
}