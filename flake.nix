{
  outputs = { self, nixpkgs }: let pkgs = nixpkgs.legacyPackages.x86_64-linux; in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        openssl
        pkgconfig
        clang
        libclang
        glibc_multi
        gcc-arm-embedded-9
      ];

      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    };
  };
}
