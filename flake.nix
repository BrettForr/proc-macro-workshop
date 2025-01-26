{
  description = "A basic Rust devshell";
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

        # Common shell configuration
        baseShell = {
          buildInputs = with pkgs; [
            openssl
            cmake
            pkg-config
            cargo-dist
            llvmPackages_latest.llvm
            llvmPackages_latest.bintools
            zlib.out
            llvmPackages_latest.lld
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
              extensions = [ "rust-src" "rustfmt" "clippy" "rust-analyzer" ];
            }))
          ];
          shellHook = ''
            alias ls=exa
            alias find=fd
            alias grep=ripgrep
          '';
        };
      in
      {
        devShells = {
          default = pkgs.mkShell baseShell;
        };
      }
    );
}
