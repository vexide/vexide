{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    cargo-pros.url = "github:vexide/cargo-pros";
    pros-cli-nix.url = "github:BattleCh1cken/pros-cli-nix";
  };

  outputs = { nixpkgs, flake-utils, cargo-pros, pros-cli-nix, ... }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        cargo-pros' = cargo-pros.packages.${system}.default;
        pros-cli = pros-cli-nix.packages.${system}.default;
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [ cargo-binutils cargo-pros' pros-cli ];
        };
      }));
}
