{
  outputs = { self, nixpkgs }: let pkgs = nixpkgs.legacyPackages.x86_64-linux; in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        unzip
        openssl
        pkgconfig
        libclang
      ];

      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    };
  };
}
