{
  inputs = {
    nixpkgs.url      = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rusttmp = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
        rust = rusttmp.override {
          extensions = [ "rust-analysis" ];
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.rust-analyzer
            rust
            pkgs.pkg-config
            pkgs.binutils
            pkgs.gcc
          ];
        };
      }
    );
}
