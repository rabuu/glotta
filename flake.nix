{
	description = "Glotta - My toy language";

	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = { self, nixpkgs, flake-utils }:
		flake-utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
				};
			in {
				devShells.default = pkgs.mkShell.override { stdenv = pkgs.clangStdenv; } {
					packages = with pkgs; [
						clang-tools
						glibc
						gnumake
						bear
						lldb
					];
				};
			});
}
