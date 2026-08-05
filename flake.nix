{
  description = "glotta - my toy language";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          name = "glotta";

          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer

            gcc
            gdb

            nasm

            just
            fd
          ];
        };
      }
    );
}
