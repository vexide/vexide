{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    cargo-v5.url = "github:vexide/cargo-v5";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";

    cargo-v5.inputs.nixpkgs.follows = "nixpkgs";
    cargo-v5.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = { nixpkgs, flake-utils, cargo-v5, rust-overlay, ... }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        cargo-v5' = cargo-v5.packages.${system}.default;
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.nightly."2024-02-07".default.override {
              extensions = [ "rust-src" "rust-analyzer" "clippy" ];
            })
            cargo-v5'
          ];
        };
      }));
}
