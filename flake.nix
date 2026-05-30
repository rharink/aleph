{
  description = "Aleph — near-lossless RAW compressor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
  let
    supportedSystems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
  in {
    devShells = forAllSystems (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      # Toolchain pinned via rust-toolchain.toml
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustToolchain

          # Cargo tools
          cargo-nextest
          cargo-deny

          # DX
          just
        ];

        # Not in nixpkgs / marked broken — install once with cargo install:
        #   cargo install cargo-llvm-cov --locked
        #   cargo install cargo-crap
        shellHook = ''
          echo "aleph dev shell ready (rust $(rustc --version | cut -d' ' -f2))"
        '';
      };
    });
  };
}
