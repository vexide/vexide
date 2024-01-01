{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    cargo-pros.url = "github:pros-rs/cargo-pros";
  };

  outputs = { nixpkgs, flake-utils, cargo-pros, ... }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        cargo-pros' = cargo-pros.packages.${system}.default;
      in {
        devShell =
          pkgs.mkShell { buildInputs = with pkgs; [ gcc-arm-embedded-9 cargo-pros' ]; };
      }));
}
