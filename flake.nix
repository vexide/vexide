{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }: (flake-utils.lib.eachDefaultSystem (system:
    let pkgs = nixpkgs.legacyPackages.${system}; in {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          openssl
          pkgconfig
          clang
          libclang
          gcc-arm-embedded-9
        ];

        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      };
    }
  ));
}
