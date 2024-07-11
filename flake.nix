{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    cargo-pros.url = "github:vexide/cargo-pros";
    pros-cli-nix.url = "github:BattleCh1cken/pros-cli-nix";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";

    cargo-pros.inputs.nixpkgs.follows = "nixpkgs";
    cargo-pros.inputs.flake-utils.follows = "flake-utils";

    pros-cli-nix.inputs.nixpkgs.follows = "nixpkgs";

  };

  outputs = { nixpkgs, flake-utils, cargo-pros, pros-cli-nix, rust-overlay, ... }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        cargo-pros' = cargo-pros.packages.${system}.default;
        pros-cli = pros-cli-nix.packages.${system}.default;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" "llvm-tools" "rust-analyzer" "clippy" ];
            })
            cargo-binutils
            cargo-pros'
            pros-cli
          ];
        };
      }));
}
